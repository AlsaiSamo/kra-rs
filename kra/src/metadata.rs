//! Types that make up file's metadata

use std::fmt::{self, Display};

use getset::Getters;
use quick_xml::{events::Event, reader::Reader as XmlReader};

use crate::helper::{
    event_get_attr, event_to_string, event_unwrap_as_doctype, event_unwrap_as_empty,
    event_unwrap_as_end, event_unwrap_as_start, get_text_between_tags, next_xml_event, parse_attr,
    push_and_parse_bool, push_and_parse_value,
};
use crate::{
    error::{MetadataErrorReason, XmlError},
    Colorspace,
};

use ordered_float::OrderedFloat as OF;

const MAINDOC_DOCTYPE: &str =
    r"DOC PUBLIC '-//KDE//DTD krita 2.0//EN' 'http://www.calligra.org/DTD/krita-2.0.dtd'";
const MAINDOC_XMLNS: &str = r"http://www.calligra.org/DTD/krita";
const DOCUMENTINFO_DOCTYPE: &str = r"document-info PUBLIC '-//KDE//DTD document-info 1.1//EN' 'http://www.calligra.org/DTD/document-info-1.1.dtd'";
const DOCUMENTINFO_XMLNS: &str = r"http://www.calligra.org/DTD/document-info";
const SYNTAX_VERSION: &str = "2.0";
const MIMETYPE: &str = "application/x-kra";

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

    //TODO: look into KraMetadataEnd for these two fields
    /// Projection background color.
    projection_background_color: String,
    /// Global assistants color.
    global_assistants_color: String,
    /// Mirror axis configuration.
    mirror_axis: MirrorAxis,
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
        //TODO: do we need to check this declaration properly?
        next_xml_event(reader)?;

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

        let profile = event_get_attr(&image_props, "profile")?;
        let name = event_get_attr(&image_props, "name")?;
        let description = event_get_attr(&image_props, "description")?;
        let colorspace = Colorspace::try_from(
            event_get_attr(&image_props, "colorspacename")?
                .unescape_value()?
                .as_ref(),
        )?;
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

/// Data at the end of `maindoc.xml`
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub(crate) struct KraMetadataEnd {
    //TODO: four base64 encoded bytes
    /// Projection background color.
    projection_background_color: String,
    //TODO: four comma delimited bytes
    /// Global assistants color.
    global_assistants_color: String,
    /// Mirror axis configuration.
    mirror_axis: MirrorAxis,
}

impl KraMetadataEnd {
    pub(crate) fn from_xml(reader: &mut XmlReader<&[u8]>) -> Result<Self, MetadataErrorReason> {
        //<ProjectionBackgroundColor ... />
        let event = next_xml_event(reader)?;
        let tag = event_unwrap_as_empty(event)?;
        let projection_background_color = parse_attr(event_get_attr(&tag, "ColorData")?)?;

        //<GlobalAssistantsColor ... />
        let event = next_xml_event(reader)?;
        let tag = event_unwrap_as_empty(event)?;
        let global_assistants_color = parse_attr(event_get_attr(&tag, "SimpleColorData")?)?;
        let mirror_axis = MirrorAxis::from_xml(reader)?;

        Ok(KraMetadataEnd {
            projection_background_color,
            global_assistants_color,
            mirror_axis,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
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

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
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

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
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
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct DocumentInfo {
    about: DocInfoAbout,
    author: DocInfoAuthor,
}

impl DocumentInfo {
    pub(crate) fn from_xml(reader: &mut XmlReader<&[u8]>) -> Result<Self, MetadataErrorReason> {
        //TODO: as with maindoc, this skips initial declaration
        let _event = next_xml_event(reader)?;

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
