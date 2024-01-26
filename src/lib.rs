//! Library for reading `.kra` files, which are created/modified by [Krita](https://krita.org/).
//!
//! It can be used for importing files into applications that wish to operate on layers
//! or metadata.
//!
//! The library uses GPL-3.0-only license, as portions of it are or will be adapted
//! from Krita's source code.
//!
//! The library is far from being finished at the current moment.

//TODO: remove feature dependency? when possible
#![feature(iterator_try_collect)]
#![warn(missing_docs)]

pub mod error;
pub mod layer;
pub mod metadata;

use std::{
    borrow::Cow,
    fmt::{self, Display},
    fs::File,
    io::Read,
    path::Path,
    str::FromStr,
};

use error::{MetadataErrorReason, ReadKraError, UnknownColorspace, UnknownLayerType, XmlError};
use getset::Getters;
use layer::{
    CommonNodeProps, FilterMaskProps, GroupLayerProps, Node, NodeProps, NodeType, PaintLayerProps,
    SelectionMaskProps,
};
use metadata::ImageMetadata;
use zip::ZipArchive;

use quick_xml::{
    events::{attributes::Attribute, BytesEnd, BytesStart, BytesText, Event},
    reader::Reader as XmlReader,
};

use crate::metadata::DocumentInfo;

/// Colorspace identifier.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum Colorspace {
    /// Default RGBA colorspace.
    RGBA,
}

impl TryFrom<&str> for Colorspace {
    type Error = UnknownColorspace;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "RGBA" => Ok(Colorspace::RGBA),
            other => Err(UnknownColorspace(other.to_owned())),
        }
    }
}

impl Default for Colorspace {
    fn default() -> Self {
        Colorspace::RGBA
    }
}

impl Display for Colorspace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Colorspace::RGBA => write!(f, "RGBA"),
        }
    }
}

/// A .kra file.
#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct KraFile {
    //TODO: allow storing the file, optionally?
    //file: Option(ZipArchive<File>),
    pub(crate) meta: ImageMetadata,
    pub(crate) doc_info: DocumentInfo,
    pub(crate) layers: Vec<Node>,
    //TODO: properties after layers
}

impl KraFile {
    /// Open and parse `.kra` file.
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self, ReadKraError> {
        let file = File::open(path)?;
        let mut zip = ZipArchive::new(file)?;

        let mimetype: Vec<u8> = zip.by_name("mimetype")?.bytes().try_collect()?;
        if mimetype.as_slice() != r"application/x-krita".as_bytes() {
            return Err(ReadKraError::MimetypeMismatch);
        }

        let mut doc_info = String::new();
        zip.by_name("documentinfo.xml")?
            .read_to_string(&mut doc_info)?;
        let mut doc_info = XmlReader::from_str(doc_info.as_str());
        doc_info.trim_text(true);
        let doc_info = DocumentInfo::from_xml(&mut doc_info)
            .map_err(|err| err.to_metadata_error("documentinfo.xml".into(), &doc_info))?;

        let mut maindoc = String::new();
        zip.by_name("maindoc.xml")?.read_to_string(&mut maindoc)?;
        let mut maindoc = XmlReader::from_str(maindoc.as_str());
        maindoc.trim_text(true);

        let meta = ImageMetadata::from_xml(&mut maindoc)
            .map_err(|err| err.to_metadata_error("maindoc.xml".into(), &maindoc))?;

        let layers = get_layers(&mut maindoc)
            .map_err(|err| err.to_metadata_error("maindoc".into(), &maindoc))?;

        //TODO: properties at the end of maindoc
        Ok(KraFile {
            meta,
            doc_info,
            layers,
        })
    }
}

// TODO: write a macro for some of them?
// These are helper functions to declutter main code
#[inline]
pub(crate) fn next_xml_event<'a>(reader: &mut XmlReader<&'a [u8]>) -> Result<Event<'a>, XmlError> {
    match reader.read_event() {
        Ok(event) => Ok(event),
        Err(what) => Err(XmlError::ParsingError(what)),
    }
}

#[inline]
pub(crate) fn event_unwrap_as_doctype<'a>(event: Event<'a>) -> Result<BytesText<'a>, XmlError> {
    match event {
        Event::DocType(event) => Ok(event),
        other => Err(XmlError::EventError(
            "a doctype".to_string(),
            event_to_string(&other)?,
        )),
    }
}

#[inline]
pub(crate) fn event_unwrap_as_start<'a>(event: Event<'a>) -> Result<BytesStart<'a>, XmlError> {
    match event {
        Event::Start(event) => Ok(event),
        other => Err(XmlError::EventError(
            "start event".to_string(),
            event_to_string(&other)?,
        )),
    }
}

#[inline]
pub(crate) fn event_unwrap_as_text<'a>(event: Event<'a>) -> Result<BytesText<'a>, XmlError> {
    match event {
        Event::Text(event) => Ok(event),
        other => Err(XmlError::EventError(
            "text event".to_string(),
            event_to_string(&other)?,
        )),
    }
}

#[inline]
pub(crate) fn event_unwrap_as_end<'a>(event: Event<'a>) -> Result<BytesEnd<'a>, XmlError> {
    match event {
        Event::End(event) => Ok(event),
        other => Err(XmlError::EventError(
            "end event".to_string(),
            event_to_string(&other)?,
        )),
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
                "text, CDATA or end event".to_owned(),
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

//Starts immed. before the required <layer> | <layer/> | <mask> | <mask/>
fn parse_layer(reader: &mut XmlReader<&[u8]>) -> Result<Node, MetadataErrorReason> {
    let event = next_xml_event(reader)?;

    // If the event is not empty, and it is not a group layer, it contains masks
    let could_contain_masks = match event {
        Event::Start(..) => true,
        _ => false,
    };

    let tag: BytesStart = match event {
        Event::Start(t) | Event::Empty(t) => t,
        other => {
            return Err(XmlError::EventError(
                "layer/mask start event".to_owned(),
                event_to_string(&other)?,
            )
            .into());
        }
    };

    let common = CommonNodeProps::parse_tag(&tag)?;

    let node_type = event_get_attr(&tag, "nodetype")?.unescape_value()?;
    let node_type = match node_type.as_ref() {
        //TODO: other node types

        // Group layers are special in that they contain other layers.
        // This makes it impossible to simply parse the tag and get all info.
        "grouplayer" => {
            let composite_op = event_get_attr(&tag, "compositeop")?;
            let collapsed = event_get_attr(&tag, "collapsed")?;
            let passthrough = event_get_attr(&tag, "passthrough")?;
            let opacity = event_get_attr(&tag, "opacity")?;
            let mut layers: Vec<Node> = Vec::new();

            //<layers>
            let event = next_xml_event(reader)?;
            event_unwrap_as_start(event)?;

            loop {
                match parse_layer(reader) {
                    Ok(layer) => layers.push(layer),
                    Err(MetadataErrorReason::XmlError(XmlError::EventError(ref a, ref b)))
                        // This assumes that we have hit </layers>
                        if (a == "layer/mask start event" && b == "layers") =>
                    {
                        break
                    }
                    //Actual error
                    Err(other) => return Err(other),
                }
            }

            //</layer>
            let event = next_xml_event(reader)?;
            event_unwrap_as_end(event)?;

            NodeType::GroupLayer(GroupLayerProps {
                composite_op: parse_attr(composite_op)?,
                collapsed: parse_bool(collapsed)?,
                passthrough: parse_bool(passthrough)?,
                opacity: parse_attr(opacity)?,
                layers,
            })
        }
        "paintlayer" => NodeType::PaintLayer(PaintLayerProps::parse_tag(&tag)?),
        "filtermask" => NodeType::FilterMask(FilterMaskProps::parse_tag(&tag)?),
        "selectionmask" => NodeType::SelectionMask(SelectionMaskProps::parse_tag(&tag)?),
        _ => {
            return Err(MetadataErrorReason::UnknownLayerType(UnknownLayerType(
                node_type.into_owned(),
            )));
        }
    };

    let masks = match (could_contain_masks, &node_type) {
        (_, NodeType::GroupLayer(_)) => None,
        (false, _) => None,
        (true, _) => Some(parse_mask(reader)?),
    };

    Ok(Node::new(
        NodeProps::from_parts(common, masks, node_type),
        None,
    ))
}

fn get_layers(reader: &mut XmlReader<&[u8]>) -> Result<Vec<Node>, MetadataErrorReason> {
    let mut layers: Vec<Node> = Vec::new();
    //<layers>
    let event = next_xml_event(reader)?;
    event_unwrap_as_start(event)?;

    loop {
        match parse_layer(reader) {
            Ok(layer) => layers.push(layer),
            Err(MetadataErrorReason::XmlError(XmlError::EventError(ref a, ref b)))
                if (a == "layer/mask start event" && b == "layers") =>
            {
                break;
            }
            //Actual error
            Err(other) => {
                println!("{:?}", other);
                return Err(other);
            }
        }
    }

    Ok(layers)
}

//TODO: this and parse_layer() share similarities that I would like to control
// together (like matching the layer type, or getting layers, which may be similar with grouplayer's).
fn parse_mask(reader: &mut XmlReader<&[u8]>) -> Result<Vec<Node>, MetadataErrorReason> {
    //<masks>
    let event = next_xml_event(reader)?;
    event_unwrap_as_start(event)?;

    let mut masks: Vec<Node> = Vec::new();

    // masks
    loop {
        match next_xml_event(reader)? {
            Event::End(tag) => {
                //</masks>
                if tag.as_ref() == "masks".as_bytes() {
                    break;
                } else {
                    return Err(MetadataErrorReason::XmlError(XmlError::EventError(
                        "masks end event".into(),
                        String::from_utf8(tag.as_ref().to_vec())?,
                    )));
                }
            }
            // a mask
            Event::Empty(tag) => {
                let common = CommonNodeProps::parse_tag(&tag)?;
                let node_type = event_get_attr(&tag, "nodetype")?.unescape_value()?;
                let node_type = match node_type.as_ref() {
                    "filtermask" => NodeType::FilterMask(FilterMaskProps::parse_tag(&tag)?),
                    "selectionmask" => {
                        NodeType::SelectionMask(SelectionMaskProps::parse_tag(&tag)?)
                    }
                    _ => {
                        return Err(MetadataErrorReason::UnknownLayerType(UnknownLayerType(
                            node_type.into_owned(),
                        )));
                    }
                };

                let props = NodeProps::from_parts(common, None, node_type);
                masks.push(Node::new(props, None))
            }
            other => {
                return Err(MetadataErrorReason::XmlError(XmlError::EventError(
                    "empty or end event".to_string(),
                    event_to_string(&other)?,
                )))
            }
        }
    }

    //</layer>
    let event = next_xml_event(reader)?;
    event_unwrap_as_end(event)?;

    Ok(masks)
}
