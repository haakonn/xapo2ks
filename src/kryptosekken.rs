use crate::money::{Currency, Money};
use derive_new::new;
use itertools::Itertools;
use std::fmt::Display;
use std::io;
use time::PrimitiveDateTime;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TxType {
    Consumption,
    Income,
    Interest, // A special case of Income to aid in processing
    TransferOut,
}

impl Display for TxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match &self {
            TxType::Consumption => "Forbruk",
            TxType::Income | TxType::Interest => "Inntekt",
            TxType::TransferOut => "Overføring-Ut",
        };
        write!(f, "{str}")
    }
}

#[derive(new, Clone, Debug, PartialEq, Eq)]
pub struct KsRow {
    time: PrimitiveDateTime,
    tx_type: TxType,
    incoming: Option<Money>,
    outgoing: Option<Money>,
    fee: Option<Money>,
    note: String,
}

/// Xapo pays interest on a daily basis for both BTC and USD, so an account with both will
/// generate two daily transactions each paying bitcoin within more or less the same second.
/// This function rationalizes this by merging them so that there is only one transaction per day.
fn merge_interest(rows: Vec<KsRow>) -> Vec<KsRow> {
    let (interest, other): (Vec<_>, Vec<_>) = rows
        .into_iter()
        .partition(|row| row.tx_type == TxType::Interest);
    interest
        .iter()
        .into_group_map_by(|row| row.time.date())
        .values()
        .map(|group| {
            if group.len() == 1 {
                group[0].clone()
            } else {
                let sum = group.iter().map(|row| row.incoming.unwrap().amount).sum();
                KsRow {
                    incoming: Some(Money::new(sum, Currency::Btc)),
                    note: "Summed daily interest".to_string(),
                    ..*group[0]
                }
            }
        })
        .chain(other)
        .collect()
}

/// Unifies BTC account and BTC savings into one.
fn unify(mut btc_account: Vec<KsRow>, btc_savings: Vec<KsRow>) -> Vec<KsRow> {
    let mut unified = merge_interest(btc_savings);
    unified.append(&mut btc_account);
    unified.sort_by_key(|row| row.time);
    unified
}

fn money_cols(money: Option<Money>) -> Vec<String> {
    money
        .map(|m| vec![m.amount.to_string(), m.currency.to_string()])
        .unwrap_or(vec![String::new(), String::new()])
}

const CSV_HEADER: [&str; 10] = [
    "Tidspunkt",
    "Type",
    "Inn",
    "Inn-Valuta",
    "Ut",
    "Ut-Valuta",
    "Gebyr",
    "Gebyr-Valuta",
    "Marked",
    "Notat",
];

fn row_to_record(row: KsRow) -> Vec<String> {
    let incoming = money_cols(row.incoming);
    let outgoing = money_cols(row.outgoing);
    let fee = money_cols(row.fee);
    // Kryptosekken does not permit '|' in the note column:
    let note = row.note.replace('|', "");
    [row.time.to_string(), row.tx_type.to_string()]
        .into_iter()
        .chain(incoming)
        .chain(outgoing)
        .chain(fee)
        .chain(["Xapo".to_string(), note])
        .collect_vec()
}

pub fn write_csv(btc_account: Vec<KsRow>, btc_savings: Vec<KsRow>) -> Result<(), csv::Error> {
    let rows = unify(btc_account, btc_savings);
    let mut wtr = csv::Writer::from_writer(io::stdout());
    wtr.write_record(CSV_HEADER)?;
    for row in rows {
        let record = row_to_record(row);
        wtr.write_record(record)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    fn date_time(date: &str) -> PrimitiveDateTime {
        crate::time::parse_date_time(&format!("{date} 10:11:12")).unwrap()
    }

    fn btc(amount: u8) -> Option<Money> {
        Some(Money::new(Decimal::from(amount), Currency::Btc))
    }

    fn row(date_time: PrimitiveDateTime, tx_type: TxType, amount: u8) -> KsRow {
        KsRow::new(date_time, tx_type, btc(amount), None, None, "".to_string())
    }

    #[inline]
    fn test_row_1() -> KsRow {
        row(date_time("2024-01-12"), TxType::Interest, 1)
    }

    #[test]
    fn interest_is_merged() {
        let row2 = row(date_time("2024-01-12"), TxType::Interest, 2);
        let row3 = row(date_time("2024-01-13"), TxType::Interest, 4);
        let row4 = row(date_time("2024-01-12"), TxType::Income, 1);
        let rows = vec![test_row_1(), row4.clone(), row2.clone(), row3.clone()];
        let merged = merge_interest(rows);
        assert_eq!(merged.len(), 3);
        assert!(merged.contains(&row4), "Non-interest was dropped by merge");
        assert!(merged
            .iter()
            .find(|row| row.tx_type == TxType::Interest
                && row.time == date_time("2024-01-12")
                && row.incoming == Some(Money::new(Decimal::from(3), Currency::Btc)))
            .is_some());
    }

    #[test]
    fn single_interest_not_merged() {
        let rows = vec![test_row_1()];
        let merged = merge_interest(rows);
        assert_eq!(
            merged[0],
            test_row_1(),
            "Single daily interest was changed by merge"
        );
    }
}
