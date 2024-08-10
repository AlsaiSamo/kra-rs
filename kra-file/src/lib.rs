//! Library for reading `.kra` files, which are created/modified by [Krita](https://krita.org/).
//!
//! It can be used for importing files into applications that wish to operate on layers
//! or metadata.
//!
//! The library uses GPL-3.0-only license.

#![warn(missing_docs)]

pub mod data;
pub mod error;
pub(crate) mod helper;
pub mod layer;
pub mod metadata;

use std::{
    collections::HashMap,
    fmt::{self, Display},
    fs::File,
    io::Read,
    path::Path,
};

use data::{NodeData, Unloaded};
use error::{
    MaskExpected, MetadataErrorReason, ReadKraError, UnknownColorspace, UnknownLayerType, XmlError,
};
use getset::Getters;
use helper::{
    event_get_attr, event_to_string, event_unwrap_as_end, event_unwrap_as_start, next_xml_event,
};
use layer::{
    CloneLayerProps, ColorizeMaskProps, CommonNodeProps, FileLayerProps, FillLayerProps,
    FilterLayerProps, FilterMaskProps, GroupLayerProps, Node, NodeType, PaintLayerProps,
    SelectionMaskProps, TransformMaskProps, TransparencyMaskProps, VectorLayerProps,
};
use metadata::{KraMetadata, KraMetadataEnd, KraMetadataStart};
use uuid::Uuid;
use zip::ZipArchive;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader as XmlReader;

use crate::metadata::DocumentInfo;

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
    file: Option<ZipArchive<File>>,
    meta: KraMetadata,
    doc_info: DocumentInfo,
    layers: Vec<Node>,
    files: HashMap<Uuid, NodeData>,
    //TODO: use `png` crate
    merged_image: Option<Vec<u8>>,
    preview: Option<Vec<u8>>,
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
        let meta_start = KraMetadataStart::from_xml(&mut maindoc)
            .map_err(|err| err.to_metadata_error("maindoc.xml".into(), &maindoc))?;

        let mut files = HashMap::new();

        let layers = get_layers(&mut maindoc, &mut files)
            .map_err(|err| err.to_metadata_error("maindoc".into(), &maindoc))?;

        let meta_end = KraMetadataEnd::from_xml(&mut maindoc)
            .map_err(|err| err.to_metadata_error("maindoc.xml".into(), &maindoc))?;

        let meta = KraMetadata::new(meta_start, meta_end);

        //TODO: at this point, we have the file and all metadata. All configured
        // loading of image files is done after this point.

        Ok(KraFile {
            file: None,
            meta,
            doc_info,
            layers,
            files,
            merged_image: None,
            preview: None,
        })
    }
}

//Starts immed. before the required <layer> | <layer/> | <mask> | <mask/>
fn parse_layer(
    reader: &mut XmlReader<&[u8]>,
    files: &mut HashMap<Uuid, NodeData>,
) -> Result<Node, MetadataErrorReason> {
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
        //TODO: finish (Selection mask) and verify
        "grouplayer" => {
            files.insert(common.uuid().to_owned(), NodeData::DoesNotExist);
            NodeType::GroupLayer(GroupLayerProps::parse_tag(&tag, reader, files)?)
        }
        "paintlayer" => {
            files.insert(
                common.uuid().to_owned(),
                NodeData::Unloaded(Unloaded::Image),
            );
            NodeType::PaintLayer(PaintLayerProps::parse_tag(&tag)?)
        }
        "filtermask" => {
            files.insert(
                common.uuid().to_owned(),
                NodeData::Unloaded(Unloaded::Filter),
            );
            NodeType::FilterMask(FilterMaskProps::parse_tag(&tag)?)
        }
        "filelayer" => {
            files.insert(common.uuid().to_owned(), NodeData::DoesNotExist);
            NodeType::FileLayer(FileLayerProps::parse_tag(&tag)?)
        }
        "adjustmentlayer" => {
            files.insert(
                common.uuid().to_owned(),
                NodeData::Unloaded(Unloaded::Filter),
            );
            NodeType::FilterLayer(FilterLayerProps::parse_tag(&tag)?)
        }
        "generatorlayer" => {
            files.insert(
                common.uuid().to_owned(),
                NodeData::Unloaded(Unloaded::Filter),
            );
            NodeType::FillLayer(FillLayerProps::parse_tag(&tag)?)
        }
        "clonelayer" => {
            files.insert(common.uuid().to_owned(), NodeData::DoesNotExist);
            NodeType::CloneLayer(CloneLayerProps::parse_tag(&tag)?)
        }
        "transparencymask" => {
            files.insert(
                common.uuid().to_owned(),
                NodeData::Unloaded(Unloaded::TransparencyMask),
            );
            NodeType::TransparencyMask(TransparencyMaskProps::new())
        }
        "transformmask" => {
            files.insert(
                common.uuid().to_owned(),
                NodeData::Unloaded(Unloaded::TransformMask),
            );
            NodeType::TransformMask(TransformMaskProps::new())
        }
        "colorizemask" => {
            files.insert(
                common.uuid().to_owned(),
                NodeData::Unloaded(Unloaded::ColorizeMask),
            );
            NodeType::ColorizeMask(ColorizeMaskProps::parse_tag(&tag)?)
        }
        "shapelayer" => {
            files.insert(
                common.uuid().to_owned(),
                NodeData::Unloaded(Unloaded::Vector),
            );
            NodeType::VectorLayer(VectorLayerProps::parse_tag(&tag)?)
        }
        "selectionmask" => {
            files.insert(
                common.uuid().to_owned(),
                NodeData::Unloaded(Unloaded::SelectionMask),
            );
            NodeType::SelectionMask(SelectionMaskProps::parse_tag(&tag)?)
        }
        _ => {
            return Err(MetadataErrorReason::UnknownLayerType(UnknownLayerType(
                node_type.into_owned(),
            )));
        }
    };

    let masks = match (could_contain_masks, &node_type) {
        (_, NodeType::GroupLayer(_)) => None,
        (false, _) => None,
        (true, _) => Some(parse_mask(reader, files)?),
    };

    Ok(Node::new(common, masks, node_type))
}

fn get_layers(
    reader: &mut XmlReader<&[u8]>,
    files: &mut HashMap<Uuid, NodeData>,
) -> Result<Vec<Node>, MetadataErrorReason> {
    let mut layers: Vec<Node> = Vec::new();
    //<layers>
    let event = next_xml_event(reader)?;
    event_unwrap_as_start(event)?;

    loop {
        match parse_layer(reader, files) {
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
    Ok(layers)
}

//TODO: this and parse_layer() share similarities that I would like to control
// together (like matching the layer type, or getting layers, which may be similar with grouplayer's).
fn parse_mask(
    reader: &mut XmlReader<&[u8]>,
    files: &mut HashMap<Uuid, NodeData>,
) -> Result<Vec<Node>, MetadataErrorReason> {
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
                    "filtermask" => {
                        files.insert(
                            common.uuid().to_owned(),
                            NodeData::Unloaded(Unloaded::Filter),
                        );
                        NodeType::FilterMask(FilterMaskProps::parse_tag(&tag)?)
                    }
                    "transparencymask" => {
                        files.insert(
                            common.uuid().to_owned(),
                            NodeData::Unloaded(Unloaded::TransparencyMask),
                        );
                        NodeType::TransparencyMask(TransparencyMaskProps::new())
                    }
                    "transformmask" => {
                        files.insert(
                            common.uuid().to_owned(),
                            NodeData::Unloaded(Unloaded::TransformMask),
                        );
                        NodeType::TransformMask(TransformMaskProps::new())
                    }
                    "colorizemask" => {
                        files.insert(
                            common.uuid().to_owned(),
                            NodeData::Unloaded(Unloaded::ColorizeMask),
                        );
                        NodeType::ColorizeMask(ColorizeMaskProps::parse_tag(&tag)?)
                    }
                    "selectionmask" => {
                        files.insert(
                            common.uuid().to_owned(),
                            NodeData::Unloaded(Unloaded::SelectionMask),
                        );
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
