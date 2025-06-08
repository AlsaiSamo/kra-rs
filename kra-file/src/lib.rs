//! Library for reading `.kra` files, which are created/modified by [Krita](https://krita.org/).
//!
//! It can be used for importing files into applications that wish to operate on layers
//! or metadata.
//!
//! The library uses GPL-3.0-only license.

#![warn(missing_docs)]

pub mod error;
pub(crate) mod helper;
pub mod layer;
pub mod metadata;
pub mod parse;

use std::{
    fmt::{self, Display},
    fs::File,
    io::Read,
    path::Path,
};

use error::{ReadKraError, UnknownColorspace};
use getset::Getters;
use layer::Node;
use metadata::{KraMetadata, KraMetadataEnd, KraMetadataStart};
use parse::{ParsingConfiguration, get_layers};
use zip::ZipArchive;

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
    // TODO: implement file loading
    // files: HashMap<Uuid, NodeData>,
    //TODO: use `png` crate if we want to view these
    //TODO: also, gate these behind an option
    merged_image: Option<Vec<u8>>,
    preview: Option<Vec<u8>>,
}

impl KraFile {
    /// Open and parse `.kra` file.
    pub fn read<P: AsRef<Path>>(path: P, conf: ParsingConfiguration) -> Result<Self, ReadKraError> {
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

        // let mut files = HashMap::new();

        let layers = get_layers(&mut maindoc, conf, false)
            .map_err(|err| err.to_metadata_error("maindoc".into(), &maindoc))?;

        let meta_end = KraMetadataEnd::from_xml(&mut maindoc)
            .map_err(|err| err.to_metadata_error("maindoc.xml".into(), &maindoc))?;

        let meta = KraMetadata::new(meta_start, meta_end);

        Ok(KraFile {
            file: None,
            meta,
            doc_info,
            layers,
            // files,
            merged_image: None,
            preview: None,
        })
    }
}
