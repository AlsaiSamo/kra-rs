//! Types that make up file's metadata

use std::{
    fmt::{self, Display},
    str,
};

use getset::Getters;
use quick_xml::name::QName;
use quick_xml::{events::Event, reader::Reader as XmlReader};

use crate::error::{MetadataErrorReason, XmlError};
use crate::helper::{
    event_get_attr, event_to_string, event_unwrap_as_doctype, event_unwrap_as_empty,
    event_unwrap_as_end, event_unwrap_as_start, get_text_between_tags, next_xml_event, parse_attr,
    push_and_parse_bool, push_and_parse_value,
};

#[cfg(not(feature = "data"))]
use crate::dummy::Colorspace;

use ordered_float::OrderedFloat as OF;

const MAINDOC_DOCTYPE: &str =
    r"DOC PUBLIC '-//KDE//DTD krita 2.0//EN' 'http://www.calligra.org/DTD/krita-2.0.dtd'";
const MAINDOC_XMLNS: &str = r"http://www.calligra.org/DTD/krita";
const DOCUMENTINFO_DOCTYPE: &str = r"document-info PUBLIC '-//KDE//DTD document-info 1.1//EN' 'http://www.calligra.org/DTD/document-info-1.1.dtd'";
const DOCUMENTINFO_XMLNS: &str = r"http://www.calligra.org/DTD/document-info";
const SYNTAX_VERSION: &str = "2.0";
const MIMETYPE: &str = "application/x-kra";

// TODO: krita's loading routine changes from time to time.
// Select a commit some 5-8 years ago and compare that to the newest ones to confirm.

/// Metadata of the image.
#[derive(Debug, PartialEq, Eq, Clone, Hash, Getters)]
#[getset(get = "pub", get_copy = "pub")]
pub struct KraMetadata {
    /// Version of Krita under which the file was saved.
    krita_version: String,
    /// Name of the image.
    name: String,
    /// Description of the image.
    description: String,
    /// Colorspace of the image.
    colorspace: Colorspace,
    /// Color profile of the image.
    profile: String,
    /// Height, in pixels.
    height: u32,
    /// Width, in pixels.
    width: u32,
    /// Dots per inch vertically.
    y_res: u32,
    /// Dots per inch horisontally.
    x_res: u32,
    // TODO: optional proofing information (starts at line 275)
    // which probably should be parsed as it relates to how the image looks on the screen

    // NOTE: these optional fields fit into KraMetadataEnd
    // (and they will not be implemented properly until Harujion
    // or some other project starts to need them, as they are out of the current scope)
    // Their order does not matter much as the loading routine is a loop over
    // open/empty events.
    /// Projection background color.
    projection_background_color: Option<String>,
    /// Global assistants color.
    global_assistants_color: Option<String>,
    // TODO: color history
    // TODO: proofing warning color
    // TODO: animation metadata
    // TODO: compositions
    // TODO: grid
    // TODO: guides
    /// Mirror axis configuration.
    mirror_axis: Option<MirrorAxis>,
    // TODO: assistants
    // TODO: audio
    // TODO: palettes
    // TODO: resources
    // TODO: annotations
}

impl Display for KraMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl KraMetadata {
    pub(crate) fn new(start: KraMetadataStart, end: KraMetadataEnd) -> KraMetadata {
        KraMetadata {
            krita_version: start.krita_version,
            name: start.name,
            description: start.description,
            colorspace: start.colorspace,
            profile: start.profile,
            height: start.height,
            width: start.width,
            y_res: start.y_res,
            x_res: start.x_res,
            projection_background_color: end.projection_background_color,
            global_assistants_color: end.global_assistants_color,
            mirror_axis: end.mirror_axis,
        }
    }
}

/// Starting portion of metadata.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub(crate) struct KraMetadataStart {
    /// Version of Krita under which the file was saved.
    krita_version: String,
    /// Name of the image.
    name: String,
    /// Description of the image.
    description: String,
    /// Colorspace of the image.
    colorspace: Colorspace,
    /// Color profile of the image.
    profile: String,
    /// Height, in pixels.
    height: u32,
    /// Width, in pixels.
    width: u32,
    /// Dots per inch vertically.
    y_res: u32,
    /// Dots per inch horisontally.
    x_res: u32,
}

impl KraMetadataStart {
    pub(crate) fn from_xml(reader: &mut XmlReader<&[u8]>) -> Result<Self, MetadataErrorReason> {
        next_xml_event(reader)?;
        // TODO: rewrite this?
        // match event {
        //     Event::Decl(decl) => {
        //         match decl.encoding() {
        //             Some(enc) => {
        //                 if enc? != b"UTF-8".as_ref() {
        //                     todo!()
        //                 }
        //             }
        //             // Assume UTF8
        //             None => {},
        //         };
        //         let what = decl.version()?.into_owned();
        //         if what != b"1.0".as_ref() {
        //             let what = String::from_utf8(what)?;
        //             return Err(MetadataErrorReason::XmlError(XmlError::AssertionFailed("1.0", what)))
        //         };
        //     }
        //     _ => todo!(),
        // };

        let event = next_xml_event(reader)?;
        let doctype = event_unwrap_as_doctype(event)?.unescape()?;
        if doctype != MAINDOC_DOCTYPE {
            return Err(MetadataErrorReason::XmlError(XmlError::AssertionFailed(
                MAINDOC_DOCTYPE,
                doctype.to_string(),
            )));
        };

        let event = next_xml_event(reader)?;
        let doc_start = event_unwrap_as_start(event)?;
        let xmlns = event_get_attr(&doc_start, "xmlns")?.unescape_value()?;
        if xmlns != MAINDOC_XMLNS {
            return Err(MetadataErrorReason::XmlError(XmlError::AssertionFailed(
                MAINDOC_XMLNS,
                xmlns.to_string(),
            )));
        };

        let syntax_version = event_get_attr(&doc_start, "syntaxVersion")?.unescape_value()?;
        if syntax_version != SYNTAX_VERSION {
            return Err(MetadataErrorReason::XmlError(XmlError::AssertionFailed(
                SYNTAX_VERSION,
                syntax_version.to_string(),
            )));
        };

        let krita_version = event_get_attr(&doc_start, "kritaVersion")?;

        let event = next_xml_event(reader)?;
        let image_props = event_unwrap_as_start(event)?;

        let mime = event_get_attr(&image_props, "mime")?.unescape_value()?;
        if mime != MIMETYPE {
            return Err(MetadataErrorReason::XmlError(XmlError::AssertionFailed(
                MIMETYPE,
                mime.to_string(),
            )));
        };

        // TODO: may not exist? Can this happen in modern Krita?
        // If not, then assume it exists.
        let profile = event_get_attr(&image_props, "profile")?;
        let name = event_get_attr(&image_props, "name")?;
        let description = event_get_attr(&image_props, "description")?;
        // NOTE: also accounts for variants listed in function convertColorSpaceNames.
        let colorspace = Colorspace::try_from(
            event_get_attr(&image_props, "colorspacename")?
                .unescape_value()?
                .as_ref(),
        )
        .unwrap_or(Colorspace::RGBA);
        let height = event_get_attr(&image_props, "height")?;
        let width = event_get_attr(&image_props, "width")?;
        let x_res = event_get_attr(&image_props, "x-res")?;
        let y_res = event_get_attr(&image_props, "y-res")?;

        Ok(KraMetadataStart {
            krita_version: krita_version.unescape_value()?.to_string(),
            name: name.unescape_value()?.to_string(),
            description: description.unescape_value()?.to_string(),
            colorspace,
            profile: profile.unescape_value()?.to_string(),
            height: parse_attr(height)?,
            width: parse_attr(width)?,
            y_res: parse_attr(y_res)?,
            x_res: parse_attr(x_res)?,
        })
    }
}

// TODO: proper types for projection background color, global asisstants color, etc.
/// Data at the end of `maindoc.xml`
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub(crate) struct KraMetadataEnd {
    //TODO: four base64 encoded bytes
    /// Projection background color.
    projection_background_color: Option<String>,
    //TODO: four comma delimited bytes
    /// Global assistants color.
    global_assistants_color: Option<String>,
    /// Mirror axis configuration.
    mirror_axis: Option<MirrorAxis>,
    // TODO: implement other things
}

impl KraMetadataEnd {
    pub(crate) fn from_xml(reader: &mut XmlReader<&[u8]>) -> Result<Self, MetadataErrorReason> {
        let mut projection_background_color = None;
        let mut global_assistants_color = None;
        let mut mirror_axis = None;

        loop {
            let event = next_xml_event(reader)?;
            match event {
                // TODO: many items are not going to be parsed until they are properly implemented
                // TODO: palettes, resources probably go into Start?
                Event::Start(tag) => match str::from_utf8(&tag)? {
                    "MirrorAxis" => {
                        // TODO: fix parsing of mirror axis, then uncomment
                        // mirror_axis = Some(MirrorAxis::from_xml(reader)?)
                        reader.read_to_end(QName("MirrorAxis".as_ref()))?;
                    }
                    "ProofingWarningColor" => {
                        reader.read_to_end(QName("ProofingWarningColor".as_ref()))?;
                    }
                    "guides" => {
                        reader.read_to_end(QName("guides".as_ref()))?;
                    }
                    "animation" => {
                        reader.read_to_end(QName("animation".as_ref()))?;
                    }
                    other => {
                        reader.read_to_end(QName(other.as_ref()))?;
                    }
                },
                Event::Empty(tag) => match str::from_utf8(&tag)? {
                    "ProjectionBackgroundColor" => {
                        projection_background_color =
                            Some(parse_attr(event_get_attr(&tag, "ColorData")?)?)
                    }
                    "GlobalAssistantsColor" => {
                        global_assistants_color =
                            Some(parse_attr(event_get_attr(&tag, "SimpleColorData")?)?)
                    }
                    _ => {}
                },
                Event::End(tag) => match str::from_utf8(&tag)? {
                    "IMAGE" => break,
                    what => {
                        return Err(MetadataErrorReason::XmlError(XmlError::EventError(
                            "</IMAGE>",
                            what.to_string(),
                        )));
                    }
                },
                what => {
                    return Err(MetadataErrorReason::XmlError(XmlError::EventError(
                        "start event, empty event or </IMAGE>",
                        event_to_string(&what)?,
                    )));
                }
            }
        }

        Ok(KraMetadataEnd {
            projection_background_color,
            global_assistants_color,
            mirror_axis,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Getters)]
#[getset(get = "pub", get_copy = "pub")]
/// Mirroring axis properties.
pub struct MirrorAxis {
    mirror_horizontal: bool,
    mirror_vertical: bool,
    lock_horizontal: bool,
    lock_vertical: bool,
    hide_horizontal_decoration: bool,
    hide_vertical_decoration: bool,

    handle_size: OF<f32>,
    horizontal_handle_position: OF<f32>,
    vertical_handle_position: OF<f32>,
    axis_position: [OF<f32>; 2],
}

impl MirrorAxis {
    pub(crate) fn from_xml(reader: &mut XmlReader<&[u8]>) -> Result<Self, MetadataErrorReason> {
        // <MirrorAxis>
        next_xml_event(reader)?;

        let mirror_horizontal = push_and_parse_bool(reader)?;
        let mirror_vertical = push_and_parse_bool(reader)?;
        let lock_horizontal = push_and_parse_bool(reader)?;
        let lock_vertical = push_and_parse_bool(reader)?;
        let hide_horizontal_decoration = push_and_parse_bool(reader)?;
        let hide_vertical_decoration = push_and_parse_bool(reader)?;

        let handle_size = push_and_parse_value(reader)?;
        let horizontal_handle_position = push_and_parse_value(reader)?;
        let vertical_handle_position = push_and_parse_value(reader)?;

        let event = next_xml_event(reader)?;
        let tag = event_unwrap_as_empty(event)?;
        let x = event_get_attr(&tag, "x")?;
        let y = event_get_attr(&tag, "y")?;

        Ok(MirrorAxis {
            mirror_horizontal,
            mirror_vertical,
            lock_horizontal,
            lock_vertical,
            hide_horizontal_decoration,
            hide_vertical_decoration,
            handle_size,
            horizontal_handle_position,
            vertical_handle_position,
            axis_position: [parse_attr(x)?, parse_attr(y)?],
        })
    }
}

/// Information about the file.
#[derive(Debug, PartialEq, Eq, Clone, Hash, Getters)]
#[getset(get = "pub", get_copy = "pub")]
pub struct DocInfoAbout {
    title: String,
    description: String,
    subject: String,
    r#abstract: String,
    keyword: String,
    initial_creator: String,
    editing_cycles: String,
    editing_time: String,
    date: String,
    creation_date: String,
    language: String,
    license: String,
}

/// Information about the author of the file.
#[derive(Debug, PartialEq, Eq, Clone, Hash, Getters)]
#[getset(get = "pub", get_copy = "pub")]
pub struct DocInfoAuthor {
    full_name: String,
    creator_first_name: String,
    creator_last_name: String,
    initial: String,
    author_title: String,
    position: String,
    company: String,
}

/// File metadata.
#[derive(Debug, PartialEq, Eq, Clone, Hash, Getters)]
#[getset(get = "pub", get_copy = "pub")]
pub struct DocumentInfo {
    about: DocInfoAbout,
    author: DocInfoAuthor,
}

impl DocumentInfo {
    pub(crate) fn from_xml(reader: &mut XmlReader<&[u8]>) -> Result<Self, MetadataErrorReason> {
        let event = next_xml_event(reader)?;
        // NOTE: similar to what maindoc parsing has (KraMetadataStart:from_xml())
        // match event {
        //     Event::Decl(decl) => {
        //         if decl.version()? != b"1.0".as_ref() {
        //             todo!()
        //         };
        //         // TODO: rewrite into unwrap_or()
        //         match decl.encoding() {
        //             Some(enc) => {
        //                 if enc? != b"UTF-8".as_ref() {
        //                     todo!()
        //                 }
        //             }
        //             None => todo!(),
        //         };
        //     }
        //     _ => todo!(),
        // };

        let event = next_xml_event(reader)?;
        let doctype = event_unwrap_as_doctype(event)?.unescape()?;
        if doctype != DOCUMENTINFO_DOCTYPE {
            return Err(MetadataErrorReason::XmlError(XmlError::AssertionFailed(
                DOCUMENTINFO_DOCTYPE,
                doctype.to_string(),
            )));
        };

        //<document-info>
        let event = next_xml_event(reader)?;
        let doc_info = event_unwrap_as_start(event)?;
        let xmlns = event_get_attr(&doc_info, "xmlns")?.unescape_value()?;
        if xmlns != DOCUMENTINFO_XMLNS {
            return Err(MetadataErrorReason::XmlError(XmlError::AssertionFailed(
                DOCUMENTINFO_XMLNS,
                xmlns.to_string(),
            )));
        };

        //<about>
        let event = next_xml_event(reader)?;
        event_unwrap_as_start(event)?;

        let title = get_text_between_tags(reader)?.to_string();
        let description = get_text_between_tags(reader)?.to_string();
        let subject = get_text_between_tags(reader)?.to_string();
        let r#abstract = get_text_between_tags(reader)?.to_string();
        let keyword = get_text_between_tags(reader)?.to_string();
        let initial_creator = get_text_between_tags(reader)?.to_string();
        let editing_cycles = get_text_between_tags(reader)?.to_string();
        let editing_time = get_text_between_tags(reader)?.to_string();
        let date = get_text_between_tags(reader)?.to_string();
        let creation_date = get_text_between_tags(reader)?.to_string();
        let language = get_text_between_tags(reader)?.to_string();
        let license = get_text_between_tags(reader)?.to_string();

        let about = DocInfoAbout {
            title,
            description,
            subject,
            r#abstract,
            keyword,
            initial_creator,
            editing_cycles,
            editing_time,
            date,
            creation_date,
            language,
            license,
        };

        //</about>
        let event = next_xml_event(reader)?;
        event_unwrap_as_end(event)?;
        //<author>
        let event = next_xml_event(reader)?;
        event_unwrap_as_start(event)?;

        let full_name = get_text_between_tags(reader)?.to_string();
        let creator_first_name = get_text_between_tags(reader)?.to_string();
        let creator_last_name = get_text_between_tags(reader)?.to_string();
        let initial = get_text_between_tags(reader)?.to_string();
        let author_title = get_text_between_tags(reader)?.to_string();
        let position = get_text_between_tags(reader)?.to_string();
        let company = get_text_between_tags(reader)?.to_string();

        let author = DocInfoAuthor {
            full_name,
            creator_first_name,
            creator_last_name,
            initial,
            author_title,
            position,
            company,
        };

        //</author>
        let event = next_xml_event(reader)?;
        event_unwrap_as_end(event)?;
        //</document-info>
        let event = next_xml_event(reader)?;
        event_unwrap_as_end(event)?;

        //EOF
        match next_xml_event(reader)? {
            Event::Eof => Ok(DocumentInfo { about, author }),
            other => Err(XmlError::AssertionFailed("end of file", event_to_string(&other)?).into()),
        }
    }
}
