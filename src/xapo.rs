use csv::StringRecord;
use rust_decimal::Decimal;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;
use time::PrimitiveDateTime;

pub struct XapoRow {
    pub(crate) time: PrimitiveDateTime,
    pub(crate) amount: Decimal,
    pub(crate) action: String,
    pub(crate) sub_description: String,
}

/// Reads a Xapo CSV file faithfully without any transformation.
/// There is a `counterparty` column which we ignore.
pub fn read_file(file: &PathBuf) -> Result<Vec<XapoRow>, ParseError> {
    let mut account_reader = csv::Reader::from_path(file)?;
    account_reader
        .records()
        .map(|record| record_to_xapo_row(record?))
        .collect()
}

fn parse_decimal(input: &str) -> Result<Decimal, rust_decimal::Error> {
    Decimal::from_str(input).or_else(|_| Decimal::from_scientific(input))
}

/// Converts a CSV record into a XapoRow.
/// Note that the CSV format changed at some point in early 2026. The columns received new names
/// and appeared in different order. We only support the new format, and we also index by
/// column number, which is vulnerable to similar future changes. A good improvement would be to
/// index by column name instead, and/or to detect new and old version.
fn record_to_xapo_row(record: StringRecord) -> Result<XapoRow, ParseError> {
    Ok(XapoRow {
        time: crate::time::parse_date_time(&record[1])?,
        amount: parse_decimal(&record[4])?,
        action: record[2].to_string(),
        sub_description: record[8].to_string(),
    })
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error(transparent)]
    Csv(#[from] csv::Error),
    #[error(transparent)]
    TimeParse(#[from] time::error::Parse),
    #[error(transparent)]
    DecimalParse(#[from] rust_decimal::Error),
}
