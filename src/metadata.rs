//! Types that make up file's metadata

use std::{
    fmt::{self, Display},
    io::Read,
    str,
    str::FromStr,
};

use quick_xml::{events::Event, reader::Reader as XmlReader};

use crate::{
    error::{MetadataErrorReason, ParseUuidError, XmlError},
    event_get_attr, event_to_string, event_unwrap_as_doctype, event_unwrap_as_end,
    event_unwrap_as_start, get_text_between_tags, next_xml_event, parse_attr, Colorspace,
};

const MAINDOC_DOCTYPE: &str =
    r"DOC PUBLIC '-//KDE//DTD krita 2.0//EN' 'http://www.calligra.org/DTD/krita-2.0.dtd'";
const MAINDOC_XMLNS: &str =
    r"http://www.calligra.org/DTD/krita";
const DOCUMENTINFO_DOCTYPE: &str =
    r"document-info PUBLIC '-//KDE//DTD document-info 1.1//EN' 'http://www.calligra.org/DTD/document-info-1.1.dtd'";
const DOCUMENTINFO_XMLNS: &str =
    r"http://www.calligra.org/DTD/document-info";
const SYNTAX_VERSION: &str = "2.0";
const MIMETYPE: &str = "application/x-kra";

/// UUID of layers, stored without dashes.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Uuid([u8; 32]);

impl FromStr for Uuid {
    type Err = ParseUuidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.is_ascii() {
            return Err(ParseUuidError(s.to_owned()));
        };
        let bytes = s.as_bytes();
        let p1 = &bytes[1..9];
        let p2 = &bytes[10..14];
        let p3 = &bytes[15..19];
        let p4 = &bytes[20..24];
        let p5 = &bytes[25..=36];
        let ret = p1
            .iter()
            .chain(p2.iter())
            .chain(p3.iter())
            .chain(p4.iter())
            .chain(p5.iter())
            .copied()
            .collect::<Vec<u8>>();
        println!("{}", String::from_utf8(ret.clone()).unwrap());
        Ok(Uuid(
            ret.try_into().map_err(|_| ParseUuidError(s.to_owned()))?,
        ))
    }
}

impl<'a> Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let u1 = str::from_utf8(&self.0[0..8]).map_err(|_| fmt::Error)?;
        let u2 = str::from_utf8(&self.0[8..12]).map_err(|_| fmt::Error)?;
        let u3 = str::from_utf8(&self.0[12..16]).map_err(|_| fmt::Error)?;
        let u4 = str::from_utf8(&self.0[16..20]).map_err(|_| fmt::Error)?;
        let u5 = str::from_utf8(&self.0[20..32]).map_err(|_| fmt::Error)?;
        write!(f, "{}-{}-{}-{}-{}", u1, u2, u3, u4, u5)
    }
}

//TODO: since image metadata is split into two parts,
// create ImageMetadataEnd with all required functions,
// then rename ImageMetadata into ImageMetadataStart,
// then add ImageMetadata that combines the two.

/// Metadata of the image.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ImageMetadata {
    /// Version of Krita under which the file was saved.
    pub(crate) krita_version: String,

    /// Name of the image.
    pub(crate) name: String,

    /// Description of the image.
    pub(crate) description: String,

    /// Colorspace of the image.
    pub(crate) colorspace: Colorspace,

    /// Color profile of the image.
    pub(crate) profile: String,

    /// Height, in pixels.
    pub(crate) height: u32,

    /// Width, in pixels.
    pub(crate) width: u32,

    /// Dots per inch vertically.
    pub(crate) y_res: u32,

    /// Dots per inch horisontally.
    pub(crate) x_res: u32,
}

impl Display for ImageMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
impl ImageMetadata {
    pub(crate) fn from_xml(reader: &mut XmlReader<&[u8]>) -> Result<Self, MetadataErrorReason> {
        //TODO: do we need to check this declaration properly?
        // Processes the first declaration
        next_xml_event(reader)?;

        //Checking that the doctype has the correct DTD
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
        //TODO: what information should I check? Procdessing that it is name DOC
        // seems redundant and useless.
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

        Ok(ImageMetadata {
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

// contains data from the end of maindoc.xml
// TODO: add proper types
pub(crate) struct ImageMetadataEnd {
    pub(crate) ProjectionBackgroundColor: String,
    pub(crate) GlobalAssistantsColor: String,
    pub(crate) MirrorAxis: MirrorAxis,
}

// TODO: add proper types
pub(crate) struct MirrorAxis {
    pub(crate) mirrorHorizontal: u32,
    pub(crate) mirrorVertical: u32,
    pub(crate) lockHorizontal: u32,
    pub(crate) lockVertical: u32,
    pub(crate) hideHorizontalDecoration: u32,
    pub(crate) hideVerticalDecoration: u32,
    pub(crate) handleSize: u32,
    pub(crate) horizontalHandlePosition: u32,
    pub(crate) verticalHandlePosition: u32,
    pub(crate) axisPosition: [u32; 2],
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct DocInfoAbout {
    title: String,
    description: String,
    subject: String,
    r#abstract: String,
    keyword: String,
    initial_creator: String,
    //TODO: as timestamps?
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
        //TODO this skips initial declaration, is this fine?
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
        let xmlns = event_get_attr(&doc_info, "xmlns")?
            .unescape_value()?;
        //TODO: there are four such blocks, two in this function and two in maindoc's from_xml().
        // Cound be abstracted away
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
            other => {
                Err(XmlError::AssertionFailed("end of file", event_to_string(&other)?).into())
            }
        }
    }
}
