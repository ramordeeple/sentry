use sentry::domain::scenario::{Rate, Scenario, ValidationError};

fn valid_rate() -> Rate {
    Rate {
        id: "R01235".to_owned(),
        num_code: "840".to_owned(),
        char_code: "USD".to_owned(),
        nominal: 1,
        name: "US Dollar".to_owned(),
        value: "93.25".to_owned(),
    }
}

fn valid_scenario() -> Scenario {
    Scenario {
        date: "29/02/2024".to_owned(),
        rates: vec![valid_rate()],
    }
}

#[test]
fn accepts_valid_scenario() {
    assert_eq!(valid_scenario().validate(), Ok(()));
}

#[test]
fn rejects_invalid_date_format_and_nonexistent_date() {
    for date in ["2024-02-29", "29/2/2024", "31/02/2024"] {
        let mut scenario = valid_scenario();
        scenario.date = date.to_owned();
        assert_eq!(scenario.validate(), Err(ValidationError::InvalidDate));
    }
}

#[test]
fn rejects_empty_rates() {
    let mut scenario = valid_scenario();
    scenario.rates.clear();
    assert_eq!(scenario.validate(), Err(ValidationError::EmptyRates));
}

#[test]
fn rejects_invalid_rate_fields() {
    let invalid_rates = [
        Rate {
            id: String::new(),
            ..valid_rate()
        },
        Rate {
            num_code: "84A".to_owned(),
            ..valid_rate()
        },
        Rate {
            char_code: "usd".to_owned(),
            ..valid_rate()
        },
        Rate {
            nominal: 0,
            ..valid_rate()
        },
        Rate {
            nominal: -1,
            ..valid_rate()
        },
        Rate {
            name: " ".to_owned(),
            ..valid_rate()
        },
        Rate {
            value: "0".to_owned(),
            ..valid_rate()
        },
        Rate {
            value: "not-a-number".to_owned(),
            ..valid_rate()
        },
    ];

    for rate in invalid_rates {
        let scenario = Scenario {
            date: "29/02/2024".to_owned(),
            rates: vec![rate],
        };
        assert_eq!(scenario.validate(), Err(ValidationError::InvalidRate));
    }
}

#[test]
fn rejects_duplicate_currency() {
    let scenario = Scenario {
        date: "29/02/2024".to_owned(),
        rates: vec![valid_rate(), valid_rate()],
    };
    assert_eq!(scenario.validate(), Err(ValidationError::DuplicateCurrency));
}
