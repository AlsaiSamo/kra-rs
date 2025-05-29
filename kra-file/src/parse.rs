use quick_xml::{
    events::{BytesStart, Event},
    Reader,
};

use crate::{
    error::{MaskExpected, MetadataErrorReason, UnknownLayerType, XmlError},
    helper::{
        event_get_attr, event_to_string, event_unwrap_as_end, event_unwrap_as_start, next_xml_event,
    },
    layer::{
        CloneLayer, CloneLayerProps, ColorizeMask, ColorizeMaskProps, CommonNodeProps, FileLayer,
        FileLayerProps, FillLayer, FillLayerProps, FilterLayer, FilterLayerProps, FilterMask,
        FilterMaskProps, GroupLayer, GroupLayerProps, Node, PaintLayer, PaintLayerProps,
        SelectionMask, SelectionMaskProps, TransformMask, TransformMaskProps, TransparencyMask,
        TransparencyMaskProps, VectorLayer, VectorLayerProps,
    },
};

// TODO: should other parsing config be present?

/// Parsing confiuration and functions
#[derive(Default, Copy, Clone)]
pub enum ShouldLoadFiles {
    #[default]
    Never,
    Always,
    Condition(fn(&Node) -> bool),
}

impl ShouldLoadFiles {
    pub fn should_load_files(&self, node: &Node) -> bool {
        match self {
            Self::Never => false,
            Self::Always => true,
            Self::Condition(func) => func(node),
        }
    }
}

#[derive(Default, Copy, Clone)]
pub struct ParsingConfiguration {
    should_load_files: ShouldLoadFiles,
    should_decode_images: bool,
    should_load_composited_images: bool,
}

//Starts immediately before the required <mask> | <mask/>
pub(crate) fn parse_mask(
    reader: &mut Reader<&[u8]>,
    // TODO: handle loading files
    conf: ParsingConfiguration,
    // files: &mut HashMap<Uuid, NodeData>,
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
                        Node::FilterMask(FilterMask::new(common, FilterMaskProps::parse_tag(&tag)?))
                    }
                    "transparencymask" => Node::TransparencyMask(TransparencyMask::new(
                        common,
                        TransparencyMaskProps::new(),
                    )),
                    "transformmask" => {
                        Node::TransformMask(TransformMask::new(common, TransformMaskProps::new()))
                    }
                    "colorizemask" => Node::ColorizeMask(ColorizeMask::new(
                        common,
                        ColorizeMaskProps::parse_tag(&tag)?,
                    )),
                    "selectionmask" => Node::SelectionMask(SelectionMask::new(
                        common,
                        SelectionMaskProps::parse_tag(&tag)?,
                    )),
                    _ => {
                        return Err(MetadataErrorReason::MaskExpected(MaskExpected(
                            node_type.into_owned(),
                        )));
                    }
                };
                masks.push(node_type)
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

//Starts immed. before the required <layer> | <layer/> | <mask> | <mask/>
pub(crate) fn parse_layer(
    reader: &mut Reader<&[u8]>,
    // TODO: handle loading files
    conf: ParsingConfiguration,
    // files: &mut HashMap<Uuid, NodeData>,
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
        // TODO: grouplayer
        "grouplayer" => {
            // todo!()
            // files.insert(common.uuid().to_owned(), NodeData::DoesNotExist);
            // NodeType::GroupLayer(GroupLayerProps::parse_tag(&tag, reader, files)?)
            Node::GroupLayer(GroupLayer::new(
                common,
                GroupLayerProps::parse_tag(&tag, reader, conf)?,
            ))
        }
        "paintlayer" => {
            Node::PaintLayer(PaintLayer::new(common, PaintLayerProps::parse_tag(&tag)?))
        }
        "filtermask" => {
            Node::FilterMask(FilterMask::new(common, FilterMaskProps::parse_tag(&tag)?))
        }
        "filelayer" => Node::FileLayer(FileLayer::new(common, FileLayerProps::parse_tag(&tag)?)),
        "adjustmentlayer" => {
            Node::FilterLayer(FilterLayer::new(common, FilterLayerProps::parse_tag(&tag)?))
        }
        "generatorlayer" => {
            Node::FillLayer(FillLayer::new(common, FillLayerProps::parse_tag(&tag)?))
        }
        "clonelayer" => {
            Node::CloneLayer(CloneLayer::new(common, CloneLayerProps::parse_tag(&tag)?))
        }
        "transparencymask" => {
            Node::TransparencyMask(TransparencyMask::new(common, TransparencyMaskProps::new()))
        }
        "transformmask" => {
            Node::TransformMask(TransformMask::new(common, TransformMaskProps::new()))
        }
        "colorizemask" => Node::ColorizeMask(ColorizeMask::new(
            common,
            ColorizeMaskProps::parse_tag(&tag)?,
        )),
        "shapelayer" => {
            Node::VectorLayer(VectorLayer::new(common, VectorLayerProps::parse_tag(&tag)?))
        }
        "selectionmask" => Node::SelectionMask(SelectionMask::new(
            common,
            SelectionMaskProps::parse_tag(&tag)?,
        )),
        _ => {
            return Err(MetadataErrorReason::UnknownLayerType(UnknownLayerType(
                node_type.into_owned(),
            )));
        }
    };

    let masks = match (could_contain_masks, &node_type) {
        // TODO: uncomment when group layers get supported
        // (_, Node::GroupLayer(_)) => None,
        (false, _) => None,
        (true, _) => Some(parse_mask(reader, conf)?),
    };

    Ok(node_type)
}

// Go over layers in the group, stopping at </layer>
pub(crate) fn get_layers(
    reader: &mut quick_xml::Reader<&[u8]>,
    // TODO: handle loading files
    conf: ParsingConfiguration,
    // files: &mut HashMap<Uuid, NodeData>,
    is_group_layer: bool,
) -> Result<Vec<Node>, MetadataErrorReason> {
    let mut layers: Vec<Node> = Vec::new();
    //<layers>
    let event = next_xml_event(reader)?;
    event_unwrap_as_start(event)?;

    loop {
        // TODO: handle loading files
        match parse_layer(reader, conf) {
            Ok(layer) => layers.push(layer),
            Err(MetadataErrorReason::XmlError(XmlError::EventError(a, ref b)))
            // This assumes that we have hit </layers>
                if (a == "layer/mask start event" && b == "layers") =>
            {
                break
            }
            //Actual error
            Err(other) => return Err(other),
        }
    }

    if is_group_layer {
        // </layer>
        let event = next_xml_event(reader)?;
        event_unwrap_as_end(event)?;
    }
    Ok(layers)
}
