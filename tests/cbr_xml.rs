use std::io;

use quick_xml::{Reader, events::Event};
use sentry::{domain::scenario::Rate, serialization::cbr_xml};

fn rate(char_code: &str, name: &str, value: &str) -> Rate {
    Rate {
        id: format!("R-{char_code}&"),
        num_code: "840".to_owned(),
        char_code: char_code.to_owned(),
        nominal: 1,
        name: name.to_owned(),
        value: value.to_owned(),
    }
}

#[test]
fn renders_well_formed_utf8_xml_and_escapes_values() -> Result<(), Box<dyn std::error::Error>> {
    let xml = cbr_xml::render("02/03/2002", &[rate("USD", "Доллар <США>", "30.1372")])?;
    let document = String::from_utf8(xml.clone())?;

    assert!(document.contains("<Value>30,1372</Value>"));
    assert!(document.contains("R-USD&amp;"));
    assert!(document.contains("Доллар &lt;США&gt;"));
    assert!(document.contains("encoding=\"UTF-8\""));

    let mut reader = Reader::from_reader(xml.as_slice());
    let mut root_seen = false;
    let mut rates_seen = 0;
    loop {
        match reader.read_event()? {
            Event::Start(element) if element.name().as_ref() == b"ValCurs" => root_seen = true,
            Event::Start(element) if element.name().as_ref() == b"Valute" => rates_seen += 1,
            Event::Eof => break,
            _ => {}
        }
    }
    assert!(root_seen);
    assert_eq!(rates_seen, 1);
    Ok(())
}

#[test]
fn orders_multiple_rates_by_currency_code() -> Result<(), Box<dyn std::error::Error>> {
    let mut euro = rate("EUR", "Euro", "26.3732");
    euro.nominal = 100;
    let xml = cbr_xml::render("02/03/2002", &[rate("USD", "US Dollar", "30.1372"), euro])?;
    let document = String::from_utf8(xml)?;
    let eur_position = document
        .find("<CharCode>EUR</CharCode>")
        .ok_or_else(|| io::Error::other("EUR is missing from XML"))?;
    let usd_position = document
        .find("<CharCode>USD</CharCode>")
        .ok_or_else(|| io::Error::other("USD is missing from XML"))?;

    assert!(eur_position < usd_position);
    assert_eq!(document.matches("<Valute ").count(), 2);
    assert!(document.contains("<Nominal>100</Nominal>"));
    Ok(())
}
