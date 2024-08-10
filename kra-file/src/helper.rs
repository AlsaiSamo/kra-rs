use std::borrow::Cow;
use std::fmt::Display;
use std::str::FromStr;

use quick_xml::events::attributes::Attribute;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Reader as XmlReader;

use crate::error::XmlError;

// These are helper functions to declutter main code
#[inline]
pub(crate) fn next_xml_event<'a>(reader: &mut XmlReader<&'a [u8]>) -> Result<Event<'a>, XmlError> {
    match reader.read_event() {
        Ok(event) => Ok(event),
        Err(what) => Err(XmlError::ParsingError(what)),
    }
}

#[inline]
pub(crate) fn event_unwrap_as_doctype(event: Event) -> Result<BytesText, XmlError> {
    match event {
        Event::DocType(event) => Ok(event),
        other => Err(XmlError::EventError("a doctype", event_to_string(&other)?)),
    }
}

#[inline]
pub(crate) fn event_unwrap_as_start(event: Event) -> Result<BytesStart, XmlError> {
    match event {
        Event::Start(event) => Ok(event),
        other => Err(XmlError::EventError(
            "start event",
            event_to_string(&other)?,
        )),
    }
}

#[inline]
pub(crate) fn event_unwrap_as_empty(event: Event) -> Result<BytesStart, XmlError> {
    match event {
        Event::Empty(event) => Ok(event),
        other => Err(XmlError::EventError(
            "start event",
            event_to_string(&other)?,
        )),
    }
}

#[inline]
pub(crate) fn event_unwrap_as_end(event: Event) -> Result<BytesEnd, XmlError> {
    match event {
        Event::End(event) => Ok(event),
        other => Err(XmlError::EventError("end event", event_to_string(&other)?)),
    }
}

#[inline]
pub(crate) fn event_get_attr<'a>(
    tag: &'a BytesStart<'a>,
    name: &str,
) -> Result<Attribute<'a>, XmlError> {
    let attr = tag
        .try_get_attribute(name)?
        .ok_or(XmlError::MissingValue(name.to_owned()))?;
    Ok(attr)
}

//Does not work on bools, use parse_bool() instead
// This is because xml data stores bools as 1/0 while parse::<bool> expects true/false
#[inline]
pub(crate) fn parse_attr<T>(attr: Attribute) -> Result<T, XmlError>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    match attr.unescape_value()?.parse::<T>() {
        Ok(item) => Ok(item),
        Err(what) => Err(XmlError::ValueError(what.to_string())),
    }
}

// parse_attr() but for bools, refer to the comment above parse_attr()
#[inline]
pub(crate) fn parse_bool(attr: Attribute) -> Result<bool, XmlError> {
    match attr.unescape_value()?.as_ref() {
        "1" => Ok(true),
        "0" => Ok(false),
        what => Err(XmlError::ValueError(what.to_string())),
    }
}

// gets next event and parses its value
#[inline]
pub(crate) fn push_and_parse_value<T>(reader: &mut XmlReader<&[u8]>) -> Result<T, XmlError>
where
    T: FromStr,
    <T as FromStr>::Err: Display,
{
    let event = next_xml_event(reader)?;
    let tag = event_unwrap_as_empty(event)?;
    let attr = event_get_attr(&tag, "value")?;
    Ok(parse_attr::<T>(attr)?)
}

//same but for bool
#[inline]
pub(crate) fn push_and_parse_bool(reader: &mut XmlReader<&[u8]>) -> Result<bool, XmlError> {
    let event = next_xml_event(reader)?;
    let tag = event_unwrap_as_empty(event)?;
    let attr = event_get_attr(&tag, "value")?;
    Ok(parse_bool(attr)?)
}

//Starts immed. before the start tag
pub(crate) fn get_text_between_tags<'a>(
    reader: &mut XmlReader<&'a [u8]>,
) -> Result<Cow<'a, str>, XmlError> {
    let event = next_xml_event(reader)?;
    event_unwrap_as_start(event)?;

    let event = next_xml_event(reader)?;
    let text = match event {
        Event::Text(text) => text.unescape()?,
        Event::CData(cdata) => cdata.escape()?.unescape()?,
        //no text -> we are at the end tag -> short circuit and don't check for the end
        Event::End(_) => {
            return Ok(Cow::Owned("".to_owned()));
        }
        other => {
            return Err(XmlError::EventError(
                "text, CDATA or end event",
                event_to_string(&other)?,
            ));
        }
    };

    let event = next_xml_event(reader)?;
    event_unwrap_as_end(event)?;

    Ok(text)
}

pub(crate) fn event_to_string(event: &Event) -> Result<String, XmlError> {
    let bytes: Vec<u8> = event.iter().copied().collect();
    Ok(String::from_utf8(bytes)?)
}
