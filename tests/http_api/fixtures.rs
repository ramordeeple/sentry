use std::io;

use chrono::NaiveDate;
use sentry::domain::scenario::{Rate, Scenario};

pub(crate) const SCENARIO_DATE: &str = "02/03/2002";
pub(crate) const SCENARIO_RATE: &str = "30.1372";

pub(crate) fn scenario(date: &str, value: &str) -> Scenario {
    Scenario {
        date: date.to_owned(),
        rates: vec![Rate {
            id: "R01235".to_owned(),
            num_code: "840".to_owned(),
            char_code: "USD".to_owned(),
            nominal: 1,
            name: "Доллар США".to_owned(),
            value: value.to_owned(),
        }],
    }
}

pub(crate) fn generated_date(ordinal: u32) -> Result<String, io::Error> {
    NaiveDate::from_yo_opt(2000, ordinal)
        .map(|date| date.format("%d/%m/%Y").to_string())
        .ok_or_else(|| io::Error::other("test generated an invalid ordinal date"))
}
