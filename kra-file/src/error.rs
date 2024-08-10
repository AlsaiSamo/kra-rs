//! Error types for the library.

use std::{io, path::PathBuf, string::FromUtf8Error};

use quick_xml::Reader;
use thiserror::Error;

//TODO: getter for the error string? for all these structs

#[derive(Error, Debug, PartialEq, Eq, Clone, Hash)]
#[error("unknown colorspace: {0}")]
pub struct UnknownColorspace(pub(crate) String);

#[derive(Error, Debug, PartialEq, Eq, Clone, Hash)]
#[error("failed to parse UUID: {0}")]
pub struct ParseUuidError(uuid::Error);

#[derive(Error, Debug, PartialEq, Eq, Clone, Hash)]
#[error("unknown compositeop: {0}")]
pub struct UnknownCompositeOp(pub(crate) String);

#[derive(Error, Debug, PartialEq, Eq, Clone, Hash)]
#[error("unknown layer type: {0}")]
pub struct UnknownLayerType(pub(crate) String);

#[derive(Error, Debug, PartialEq, Eq, Clone, Hash)]
#[error("expected a mask, got: {0}")]
pub struct MaskExpected(pub(crate) String);

//TODO: instead of tuples, types? To make the inner strings un-editable

#[derive(Debug, Clone, Error)]
pub enum XmlError {
    // Error used when we check some property
    #[error("assertion about XML metadata failed: expected {0}, got {1}")]
    AssertionFailed(&'static str, String),

    // Error when XML cannot be interpreted
    #[error("could not parse XML")]
    ParsingError(#[from] quick_xml::Error),

    // Error when XML can be parsed but is incorrect
    #[error("unexpected XML event: expected {0}, got {1}")]
    EventError(&'static str, String),

    #[error("missing XML value: {0}")]
    MissingValue(String),

    // Error when XML is valid but the value contained (text or attribute) is not
    #[error("could not interpret XML value: {0}")]
    ValueError(String),

    #[error("could not interpret string as utf-8: {0}")]
    EncodingError(#[from] FromUtf8Error),
}

// Whatever error was thrown while parsing metadata
#[derive(Error, Debug)]
pub(crate) enum MetadataErrorReason {
    #[error(transparent)]
    UnknownColorspace(#[from] UnknownColorspace),

    #[error(transparent)]
    UnknownLayerType(#[from] UnknownLayerType),

    #[error(transparent)]
    MaskExpected(#[from] MaskExpected),

    #[error(transparent)]
    ParseUuidError(#[from] ParseUuidError),

    #[error(transparent)]
    XmlError(#[from] XmlError),
}

impl From<quick_xml::Error> for MetadataErrorReason {
    fn from(value: quick_xml::Error) -> Self {
        MetadataErrorReason::XmlError(XmlError::ParsingError(value))
    }
}

impl From<FromUtf8Error> for MetadataErrorReason {
    fn from(value: FromUtf8Error) -> Self {
        MetadataErrorReason::XmlError(XmlError::EncodingError(value))
    }
}

impl From<uuid::Error> for MetadataErrorReason {
    fn from(value: uuid::Error) -> Self {
        MetadataErrorReason::ParseUuidError(ParseUuidError(value))
    }
}

impl MetadataErrorReason {
    // Fills out MetadataError with the given reason and location
    pub(crate) fn to_metadata_error(self, file: PathBuf, reader: &Reader<&[u8]>) -> MetadataError {
        MetadataError {
            file,
            buffer_pos: reader.buffer_position(),
            error: self,
        }
    }
}

//TODO: can buffer_pos be used to find line and column in the xml file?
// 1. Make MetadataError
// 2. Get a reader at the start of the file
// 3. Find the location

// Error that was thrown while parsing metadata, along with its location
#[derive(Error, Debug)]
#[error("{file} at {buffer_pos}: {error}")]
pub struct MetadataError {
    //TODO: could be static? Or could be reused for parsing files in general, then
    // it'll have to be nonstatic
    file: PathBuf,
    buffer_pos: usize,
    error: MetadataErrorReason,
}

/// Errors that can be encountered while opening the file.
#[derive(Error, Debug)]
pub enum ReadKraError {
    #[error(transparent)]
    FileError(#[from] io::Error),

    #[error(transparent)]
    ZipError(#[from] zip::result::ZipError),

    #[error("mimetype not recognised")]
    MimetypeMismatch,

    #[error(transparent)]
    MetadataError(#[from] MetadataError),
}
