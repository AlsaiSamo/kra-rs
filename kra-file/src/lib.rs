//! Library for reading `.kra` files, which are created/modified by [Krita](https://krita.org/).
//!
//! It can be used for importing files into applications that wish to operate on layers
//! or metadata.
//!
//! The library uses GPL-3.0-only license.

#![warn(missing_docs)]

#[cfg(not(feature = "data"))]
pub mod dummy;
pub mod error;
pub(crate) mod helper;
pub mod layer;
pub mod metadata;
pub mod parse;

use std::{fs::File, io::Read, path::Path};

use error::ReadKraError;
use getset::Getters;
use layer::Node;
use metadata::{KraMetadata, KraMetadataEnd, KraMetadataStart};
use parse::{ParsingConfiguration, get_layers};
use zip::ZipArchive;

use quick_xml::Reader as XmlReader;

use crate::metadata::DocumentInfo;

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
    // TODO: implement these (PNG images)
    // merged_image: Option<Vec<u8>>,
    // preview: Option<Vec<u8>>,
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

        let mut doc_info_data = String::new();
        zip.by_name("documentinfo.xml")?
            .read_to_string(&mut doc_info_data)?;
        let mut doc_info = XmlReader::from_str(doc_info_data.as_str());

        doc_info.config_mut().trim_text(true);
        let doc_info = DocumentInfo::from_xml(&mut doc_info).map_err(|err| {
            err.to_metadata_error(
                "documentinfo.xml".into(),
                &doc_info,
                doc_info_data.as_bytes(),
            )
        })?;

        let mut maindoc_data = String::new();
        zip.by_name("maindoc.xml")?
            .read_to_string(&mut maindoc_data)?;
        let mut maindoc = XmlReader::from_str(maindoc_data.as_str());

        maindoc.config_mut().trim_text(true);
        let meta_start = KraMetadataStart::from_xml(&mut maindoc).map_err(|err| {
            err.to_metadata_error("maindoc.xml".into(), &maindoc, maindoc_data.as_bytes())
        })?;

        // let mut files = HashMap::new();

        let layers = get_layers(&mut maindoc, conf, false).map_err(|err| {
            err.to_metadata_error("maindoc".into(), &maindoc, maindoc_data.as_bytes())
        })?;

        let meta_end = KraMetadataEnd::from_xml(&mut maindoc).map_err(|err| {
            err.to_metadata_error("maindoc.xml".into(), &maindoc, maindoc_data.as_bytes())
        })?;

        let meta = KraMetadata::new(meta_start, meta_end);

        Ok(KraFile {
            file: None,
            meta,
            doc_info,
            layers,
            // files,
            // merged_image: None,
            // preview: None,
        })
    }
}
