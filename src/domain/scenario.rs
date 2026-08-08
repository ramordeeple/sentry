use std::collections::HashSet;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct Scenario {
    pub date: String,
    pub rates: Vec<Rate>,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Rate {
    pub id: String,
    pub num_code: String,
    pub char_code: String,
    pub nominal: i64,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("date must be a valid calendar date in DD/MM/YYYY format")]
    InvalidDate,
    #[error("rates must not be empty")]
    EmptyRates,
    #[error("rate must have a 3-letter uppercase char_code, positive nominal and value")]
    InvalidRate,
    #[error("char_code must be unique within a scenario")]
    DuplicateCurrency,
}

impl Scenario {
    /// Checks invariants required before a scenario can be persisted.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if !is_valid_cbr_date(&self.date) {
            return Err(ValidationError::InvalidDate);
        }
        if self.rates.is_empty() {
            return Err(ValidationError::EmptyRates);
        }
        if self.rates.iter().any(Rate::is_invalid) {
            return Err(ValidationError::InvalidRate);
        }
        let unique_codes: HashSet<_> = self.rates.iter().map(|rate| &rate.char_code).collect();
        if unique_codes.len() != self.rates.len() {
            return Err(ValidationError::DuplicateCurrency);
        }
        Ok(())
    }
}

impl Rate {
    fn is_invalid(&self) -> bool {
        self.nominal <= 0
            || self.id.trim().is_empty()
            || self.name.trim().is_empty()
            || self.num_code.len() != 3
            || !self.num_code.bytes().all(|byte| byte.is_ascii_digit())
            || self.char_code.len() != 3
            || !self.char_code.bytes().all(|byte| byte.is_ascii_uppercase())
            || self
                .value
                .parse::<f64>()
                .map_or(true, |value| !value.is_finite() || value <= 0.0)
    }
}

fn is_valid_cbr_date(value: &str) -> bool {
    value.len() == 10
        && value.as_bytes()[2] == b'/'
        && value.as_bytes()[5] == b'/'
        && NaiveDate::parse_from_str(value, "%d/%m/%Y").is_ok()
}
