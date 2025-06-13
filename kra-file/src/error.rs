//! Error types for the library.

use std::{
    io::{self, BufRead, Cursor},
    path::PathBuf,
    str::Utf8Error,
    string::FromUtf8Error,
};

use quick_xml::{Reader, encoding::EncodingError, events::attributes::AttrError};
use thiserror::Error;

// TODO: getters for error strings?
// NOTE: all errors currently operate on owned strings, so all errors have to be cloned.
// TODO: can this be avoided, to not clone when possible?

/// The colorspace is unknown.
#[derive(Error, Debug, PartialEq, Eq, Clone, Hash)]
#[error("unknown colorspace: {0}")]
pub struct UnknownColorspace(pub(crate) String);

/// The UUID could not be parsed.
#[derive(Error, Debug, PartialEq, Eq, Clone, Hash)]
#[error("failed to parse UUID: {0}")]
pub struct ParseUuidError(uuid::Error);

/// The composition operation is unknown.
#[derive(Error, Debug, PartialEq, Eq, Clone, Hash)]
#[error("unknown compositeop: {0}")]
pub struct UnknownCompositeOp(pub(crate) String);

/// The layer type is unknown.
#[derive(Error, Debug, PartialEq, Eq, Clone, Hash)]
#[error("unknown layer type: {0}")]
pub struct UnknownLayerType(pub(crate) String);

/// A mask was expected instead of a layer.
#[derive(Error, Debug, PartialEq, Eq, Clone, Hash)]
#[error("expected a mask, got: {0}")]
pub struct MaskExpected(pub(crate) String);

#[derive(Debug, Clone, Error)]
pub enum XmlError {
    /// Error used when we check that a string in the file
    /// is equal to the expected string.
    #[error("assertion about XML metadata failed: expected {0}, got {1}")]
    AssertionFailed(&'static str, String),

    /// XML cannot be interpreted.
    #[error("could not parse XML")]
    ParsingError(#[from] quick_xml::Error),

    /// XML event can be parsed but it is not what we expect.
    #[error("unexpected XML event: expected {0}, got {1}")]
    EventError(&'static str, String),

    /// A value that we expected to exist is missing.
    #[error("missing XML value: {0}")]
    MissingValue(String),

    /// XML is valid but the value contained (text or attribute)
    /// is of unexpected type.
    #[error("could not interpret XML value: {0}")]
    ValueError(String),

    /// XML is not a valid UTF-8.
    #[error("could not interpret string as utf-8: {0}")]
    EncodingError(#[from] Utf8Error),
}

impl From<FromUtf8Error> for XmlError {
    fn from(value: FromUtf8Error) -> Self {
        XmlError::EncodingError(value.utf8_error())
    }
}

impl From<AttrError> for XmlError {
    fn from(value: AttrError) -> Self {
        XmlError::ParsingError(quick_xml::Error::InvalidAttr(value))
    }
}

impl From<EncodingError> for XmlError {
    fn from(value: EncodingError) -> Self {
        XmlError::ParsingError(quick_xml::Error::Encoding(value))
    }
}

/// The error that occured when parsing metadata.
#[derive(Error, Debug)]
pub(crate) enum MetadataErrorReason {
    /// The colorspace is unknown.
    #[error(transparent)]
    UnknownColorspace(#[from] UnknownColorspace),

    /// The layer type is unknown.
    #[error(transparent)]
    UnknownLayerType(#[from] UnknownLayerType),

    /// A mask was expected instead of a layer.
    #[error(transparent)]
    MaskExpected(#[from] MaskExpected),

    /// The value cannot be parsed as UUID.
    #[error(transparent)]
    ParseUuidError(#[from] ParseUuidError),

    /// Error in parsing XML.
    #[error(transparent)]
    XmlError(#[from] XmlError),
}

impl From<quick_xml::Error> for MetadataErrorReason {
    fn from(value: quick_xml::Error) -> Self {
        MetadataErrorReason::XmlError(XmlError::ParsingError(value))
    }
}

impl From<AttrError> for MetadataErrorReason {
    fn from(value: AttrError) -> Self {
        MetadataErrorReason::XmlError(XmlError::from(value))
    }
}

impl From<Utf8Error> for MetadataErrorReason {
    fn from(value: Utf8Error) -> Self {
        MetadataErrorReason::XmlError(XmlError::EncodingError(value))
    }
}

impl From<FromUtf8Error> for MetadataErrorReason {
    fn from(value: FromUtf8Error) -> Self {
        MetadataErrorReason::XmlError(XmlError::EncodingError(value.utf8_error()))
    }
}

impl From<uuid::Error> for MetadataErrorReason {
    fn from(value: uuid::Error) -> Self {
        MetadataErrorReason::ParseUuidError(ParseUuidError(value))
    }
}

// NOTE: Reader::read_event() is not implemented for Reader<Cursor<&[u8]>>.
// And Reader<&[u8]> does not return the complete slice that it is given, only
// what is not read.
// TODO: find how to get the complete slice from Reader
impl MetadataErrorReason {
    // Fills out MetadataError with the given reason and location
    pub(crate) fn to_metadata_error(
        self,
        file: PathBuf,
        reader: &Reader<&[u8]>,
        data: &[u8],
    ) -> MetadataError {
        let buffer_pos = reader.buffer_position();
        let mut cursor = Cursor::new(data);
        let mut last_line_length = 0;
        let mut line_num = 0;
        while cursor.position() < buffer_pos {
            // TODO: rewrite unwrap to bubble up IO errors (also add IO errors to MetadataErrorReason)
            // TODO: I do not like the try_into(), can it be avoided?
            last_line_length = cursor.skip_until(0xA).unwrap().try_into().unwrap();
            line_num += 1;
        }
        MetadataError {
            file,
            buffer_pos,
            line: line_num,
            column: last_line_length - (cursor.position() - buffer_pos),
            error: self,
        }
    }
}

/// Error that was thrown while parsing metadata in the given file and its location.
#[derive(Error, Debug)]
#[error("{file} at byte {buffer_pos} (line: {line}, column: {column}): {error}")]
pub struct MetadataError {
    file: PathBuf,
    buffer_pos: u64,
    line: u64,
    column: u64,
    error: MetadataErrorReason,
}

/// Errors that can be encountered while opening the file.
#[derive(Error, Debug)]
pub enum ReadKraError {
    /// IO error.
    #[error(transparent)]
    IOError(#[from] io::Error),

    /// Error opening the file as ZIP archive.
    #[error(transparent)]
    ZipError(#[from] zip::result::ZipError),

    /// Wrong mimetype.
    #[error("mimetype not recognised")]
    MimetypeMismatch,

    /// Error parsing metadata.
    #[error(transparent)]
    MetadataError(#[from] MetadataError),
}
