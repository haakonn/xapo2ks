//! Some trivial constructs for handling money. There are third-party crates for this, but they
//! cover a lot more than we need and add a lot of complexity.
use derive_new::new;
use rust_decimal::Decimal;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use thiserror::Error;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Currency {
    Usd,
    Btc,
}

impl FromStr for Currency {
    type Err = MoneyError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_uppercase().as_str() {
            "USD" => Ok(Currency::Usd),
            "BTC" => Ok(Currency::Btc),
            unknown => Err(MoneyError::UnknownCurrency(unknown.to_string())),
        }
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match &self {
            Currency::Usd => "USD",
            Currency::Btc => "BTC",
        };
        write!(f, "{str}")
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, new)]
pub struct Money {
    pub amount: Decimal,
    pub currency: Currency,
}

#[derive(Error, Debug)]
pub enum MoneyError {
    #[error("Unknown currency {0}")]
    UnknownCurrency(String),
}
