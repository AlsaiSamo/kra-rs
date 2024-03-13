//! Library for reading `.kra` files, which are created/modified by [Krita](https://krita.org/).
//!
//! It can be used for importing files into applications that wish to operate on layers
//! or metadata.
//!
//! The library uses GPL-3.0-only license, as portions of it are or will be adapted
//! from Krita's source code.
//!
//! The library is far from being finished at the current moment.

#![warn(missing_docs)]

pub mod data;
pub mod error;
pub(crate) mod helper;
pub mod layer;
pub mod metadata;

use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::{self, Display},
    fs::File,
    io::Read,
    path::Path,
    str::FromStr,
};

use data::NodeData;
use error::{
    MaskExpected, MetadataErrorReason, ReadKraError, UnknownColorspace, UnknownLayerType, XmlError,
};
use getset::Getters;
use helper::{
    event_get_attr, event_to_string, event_unwrap_as_end, event_unwrap_as_start, next_xml_event,
};
use layer::{
    CommonNodeProps, FilterMaskProps, GroupLayerProps, Node, NodeType, PaintLayerProps,
    SelectionMaskProps,
};
use metadata::{ImageMetadata, ImageMetadataEnd};
use uuid::Uuid;
use zip::ZipArchive;

use quick_xml::{
    events::{attributes::Attribute, BytesEnd, BytesStart, BytesText, Event},
    reader::Reader as XmlReader,
};

use crate::metadata::DocumentInfo;

//TODO: move out?
/// Colorspace identifier.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
#[derive(Default)]
pub enum Colorspace {
    /// Default RGBA colorspace.
    #[default]
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
    pub(crate) file: Option<ZipArchive<File>>,
    pub(crate) meta: ImageMetadata,
    pub(crate) meta_end: ImageMetadataEnd,
    pub(crate) doc_info: DocumentInfo,
    pub(crate) layers: Vec<Node>,
    pub(crate) files: HashMap<Uuid, NodeData>,
    //TODO: mergedimage and preview
}

impl KraFile {
    //TODO: the function should load all files except mergedimage and preview,
    // and including file layers, and does not store the file.
    // TODO: builder for customised read()
    // TODO: mention all of this in the documentation.
    /// Open and parse `.kra` file.
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self, ReadKraError> {
        let file = File::open(path)?;
        let mut zip = ZipArchive::new(file)?;

        //Replacement of try_collect(), which is unstable
        let mimetype: Vec<u8> = zip
            .by_name("mimetype")?
            .bytes()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
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

        let meta_end = ImageMetadataEnd::from_xml(&mut maindoc)
            .map_err(|err| err.to_metadata_error("maindoc.xml".into(), &maindoc))?;

        Ok(KraFile {
            file: None,
            meta,
            meta_end,
            doc_info,
            layers,
            //TODO: fill out
            files: HashMap::new(),
        })
    }
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
            return Err(
                XmlError::EventError("layer/mask start event", event_to_string(&other)?).into(),
            );
        }
    };

    let common = CommonNodeProps::parse_tag(&tag)?;

    let node_type = event_get_attr(&tag, "nodetype")?.unescape_value()?;
    let node_type = match node_type.as_ref() {
        "grouplayer" => NodeType::GroupLayer(GroupLayerProps::parse_tag(&tag, reader)?),
        "paintlayer" => NodeType::PaintLayer(PaintLayerProps::parse_tag(&tag)?),
        "filtermask" => NodeType::FilterMask(FilterMaskProps::parse_tag(&tag)?),
        "filelayer" => NodeType::FileLayer(FileLayerProps::parse_tag(&tag)?),
        "adjustmentlayer" => NodeType::FilterLayer(FilterLayerProps::parse_tag(&tag)?),
        "generatorlayer" => NodeType::FillLayer(FillLayerProps::parse_tag(&tag)?),
        "clonelayer" => NodeType::CloneLayer(CloneLayerProps::parse_tag(&tag)?),
        "transparencymask" => NodeType::TransparencyMask(TransparencyMaskProps::new()),
        "transformmask" => NodeType::TransformMask(TransformMaskProps::new()),
        "colorizemask" => NodeType::ColorizeMask(ColorizeMaskProps::parse_tag(&tag)?),
        "shapelayer" => NodeType::VectorLayer(VectorLayerProps::parse_tag(&tag)?),
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

    Ok(Node::new(common, masks, node_type))
}

fn get_layers(reader: &mut XmlReader<&[u8]>) -> Result<Vec<Node>, MetadataErrorReason> {
    let mut layers: Vec<Node> = Vec::new();
    //<layers>
    let event = next_xml_event(reader)?;
    event_unwrap_as_start(event)?;

    loop {
        match parse_layer(reader) {
            Ok(layer) => layers.push(layer),
            Err(MetadataErrorReason::XmlError(XmlError::EventError(a, ref b)))
                //</layers>
                if (a == "layer/mask start event" && b == "layers") =>
            {
                break;
            }
            //Actual error
            Err(other) => {
                return Err(other);
            }
        }
    }

    //</layers> is already handled in the loop

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
                        "masks end event",
                        String::from_utf8(tag.as_ref().to_vec())?,
                    )));
                }
            }
            Event::Empty(tag) => {
                let common = CommonNodeProps::parse_tag(&tag)?;
                let node_type = event_get_attr(&tag, "nodetype")?.unescape_value()?;
                let node_type = match node_type.as_ref() {
                    "filtermask" => NodeType::FilterMask(FilterMaskProps::parse_tag(&tag)?),
                    "transparencymask" => NodeType::TransparencyMask(TransparencyMaskProps::new()),
                    "transformmask" => NodeType::TransformMask(TransformMaskProps::new()),
                    "colorizemask" => NodeType::ColorizeMask(ColorizeMaskProps::parse_tag(&tag)?),
                    "selectionmask" => {
                        NodeType::SelectionMask(SelectionMaskProps::parse_tag(&tag)?)
                    }
                    _ => {
                        return Err(MetadataErrorReason::MaskExpected(MaskExpected(
                            node_type.into_owned(),
                        )));
                    }
                };
                masks.push(Node::new(common, None, node_type))
            }
            other => {
                return Err(MetadataErrorReason::XmlError(XmlError::EventError(
                    "empty or end event",
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
