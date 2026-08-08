use quick_xml::{
    Writer,
    events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event},
};

use crate::domain::scenario::Rate;

pub const CONTENT_TYPE: &str = "application/xml; charset=utf-8";
const XML_VERSION: &str = "1.0";
const ENCODING: &str = "UTF-8";
const BASE_DOCUMENT_CAPACITY: usize = 256;
const ESTIMATED_RATE_CAPACITY: usize = 160;
const JSON_DECIMAL_SEPARATOR: char = '.';
const CBR_DECIMAL_SEPARATOR: &str = ",";
const ROOT_ELEMENT: &str = "ValCurs";
const RATE_ELEMENT: &str = "Valute";
const NUM_CODE_ELEMENT: &str = "NumCode";
const CHAR_CODE_ELEMENT: &str = "CharCode";
const NOMINAL_ELEMENT: &str = "Nominal";
const NAME_ELEMENT: &str = "Name";
const VALUE_ELEMENT: &str = "Value";
const DATE_ATTRIBUTE: &str = "Date";
const NAME_ATTRIBUTE: &str = "name";
const ID_ATTRIBUTE: &str = "ID";
const ROOT_NAME: &str = "Foreign Currency Market";

pub fn render(date: &str, rates: &[Rate]) -> Result<Vec<u8>, quick_xml::Error> {
    let capacity = BASE_DOCUMENT_CAPACITY + rates.len() * ESTIMATED_RATE_CAPACITY;
    let mut writer = Writer::new(Vec::with_capacity(capacity));
    writer.write_event(Event::Decl(BytesDecl::new(
        XML_VERSION,
        Some(ENCODING),
        None,
    )))?;

    let mut root = BytesStart::new(ROOT_ELEMENT);
    root.push_attribute((DATE_ATTRIBUTE, date));
    root.push_attribute((NAME_ATTRIBUTE, ROOT_NAME));
    writer.write_event(Event::Start(root))?;

    let mut ordered_rates: Vec<_> = rates.iter().collect();
    ordered_rates.sort_unstable_by_key(|rate| &rate.char_code);
    for rate in ordered_rates {
        write_rate(&mut writer, rate)?;
    }

    writer.write_event(Event::End(BytesEnd::new(ROOT_ELEMENT)))?;
    Ok(writer.into_inner())
}

fn write_rate(writer: &mut Writer<Vec<u8>>, rate: &Rate) -> Result<(), quick_xml::Error> {
    let mut element = BytesStart::new(RATE_ELEMENT);
    element.push_attribute((ID_ATTRIBUTE, rate.id.as_str()));
    writer.write_event(Event::Start(element))?;

    write_text_element(writer, NUM_CODE_ELEMENT, &rate.num_code)?;
    write_text_element(writer, CHAR_CODE_ELEMENT, &rate.char_code)?;
    write_text_element(writer, NOMINAL_ELEMENT, &rate.nominal.to_string())?;
    write_text_element(writer, NAME_ELEMENT, &rate.name)?;
    write_text_element(
        writer,
        VALUE_ELEMENT,
        &rate
            .value
            .replace(JSON_DECIMAL_SEPARATOR, CBR_DECIMAL_SEPARATOR),
    )?;

    writer.write_event(Event::End(BytesEnd::new(RATE_ELEMENT)))?;
    Ok(())
}

fn write_text_element(
    writer: &mut Writer<Vec<u8>>,
    name: &str,
    value: &str,
) -> Result<(), quick_xml::Error> {
    writer.write_event(Event::Start(BytesStart::new(name)))?;
    writer.write_event(Event::Text(BytesText::new(value)))?;
    writer.write_event(Event::End(BytesEnd::new(name)))?;
    Ok(())
}
