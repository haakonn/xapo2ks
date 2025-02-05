use crate::kryptosekken::{KsRow, TxType};
use crate::money::{Currency, Money, MoneyError};
use crate::xapo::XapoRow;
use thiserror::Error;

/// Translates from Xapo to Kryptosekken, applying all necessary transformations in the process.
pub fn xapo_to_ks(xapo_rows: Vec<XapoRow>) -> Result<Vec<KsRow>, ConvertError> {
    xapo_rows
        .into_iter()
        .map(xapo_row_to_ks)
        .filter_map(Result::transpose)
        .collect()
}

fn xapo_row_to_ks(xapo_row: XapoRow) -> Result<Option<KsRow>, ConvertError> {
    match xapo_row.description.as_str() {
        "Lightning network transaction" | "Sent BTC" => Some(xfer_out(xapo_row)).transpose(),
        "Daily USD interest" | "Daily BTC interest" => Some(interest_tx(xapo_row)).transpose(),
        "Card Cashback Redemption" => Some(cashback_tx(xapo_row)).transpose(),
        x if x.starts_with("Move ") => Ok(None),
        x if x.starts_with("Exchange ") => Some(consumption_tx(xapo_row)).transpose(),
        unknown => Err(ConvertError::UnknownTxType(unknown.to_string())),
    }
}

fn description_words(xapo_row: &XapoRow) -> Vec<&str> {
    xapo_row.description.split_whitespace().collect()
}

fn consumption_tx(xapo_row: XapoRow) -> Result<KsRow, ConvertError> {
    let descr_words = description_words(&xapo_row);
    let from_cur = descr_words[1].parse::<Currency>()?;
    let to_cur = descr_words[3].parse::<Currency>()?;
    if from_cur != Currency::Btc && to_cur != Currency::Usd {
        return Err(ConvertError::Unsupported(format!(
            "Trade from {from_cur} to {to_cur}"
        )));
    }
    Ok(KsRow::new(
        xapo_row.time,
        TxType::Consumption,
        None,
        btc(&xapo_row),
        None,
        xapo_row.sub_description,
    ))
}

fn btc(xapo_row: &XapoRow) -> Option<Money> {
    Some(Money::new(xapo_row.amount.abs(), Currency::Btc))
}

fn income_tx(xapo_row: &XapoRow, note: String, tx_type: TxType) -> Result<KsRow, ConvertError> {
    Ok(KsRow::new(
        xapo_row.time,
        tx_type,
        btc(xapo_row),
        None,
        None,
        note,
    ))
}

fn interest_tx(xapo_row: XapoRow) -> Result<KsRow, ConvertError> {
    let currency_code = xapo_row
        .description
        .split_whitespace()
        .collect::<Vec<&str>>()[1];
    income_tx(
        &xapo_row,
        format!("{} {}", currency_code, xapo_row.sub_description),
        TxType::Interest,
    )
}

fn cashback_tx(xapo_row: XapoRow) -> Result<KsRow, ConvertError> {
    let note = format!("Cashback ({})", xapo_row.sub_description);
    income_tx(&xapo_row, note, TxType::Income)
}

/// There is not enough information in Xapo's reports to determine whether an outgoing
/// transaction was a consumption event or simply a transfer between two wallets the client
/// controls. For this reason, we mark them as "transfer out" and expect the user to adjust
/// this in the CSV output.
fn xfer_out(xapo_row: XapoRow) -> Result<KsRow, ConvertError> {
    let note = format!("CHECK! sub_descr={}", xapo_row.sub_description);
    Ok(KsRow::new(
        xapo_row.time,
        TxType::TransferOut,
        None,
        btc(&xapo_row),
        None,
        note,
    ))
}

#[derive(Error, Debug)]
pub enum ConvertError {
    #[error("Unable to parse transaction type for description value {0}")]
    UnknownTxType(String),
    #[error(transparent)]
    CurrencyParse(#[from] MoneyError),
    #[error("Unsupported: {0}")]
    Unsupported(String),
}
