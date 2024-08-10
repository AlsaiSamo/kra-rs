//! Types of layer/mask data.

use core::fmt;
use std::fmt::{Debug, Display};

//TODO: store actual data
/// Data that the node refers to via `filename` property.
pub enum NodeData {
    /// Data does not exist (true for clone layers and file layers).
    DoesNotExist,
    /// Data is not loaded (yet).
    Unloaded(Unloaded),
    /// Loaded data
    Loaded(Loaded),
}

pub enum Unloaded {
    /// A compressed image.
    Image,
    /// Vector data.
    Vector,
    /// A filter configuration.
    Filter,
    /// Colorize mask data.
    ColorizeMask,
    /// Transformation mask data.
    TransformMask,
    /// Transparency mask data.
    TransparencyMask,
    /// Selection mask data.
    SelectionMask,
}

// If data handling is disabled, this type acts as a plug. The user will not be able
// to look into the NodeData::Loaded.
//
// If the user creates a library that only works with metadata, and sets "no_data",
// the code they will write will work the same as with "no_data" unset.
// TODO: remind the users to allow unsetting "no_data" feature.
#[cfg(feature = "no_data")]
pub(crate) struct Loaded();

/// Loaded data.
#[cfg(not(feature = "no_data"))]
pub enum Loaded {
    //TODO: images can be compressed and uncompressed; represent both.
    /// Raster data.
    Image,
    /// Vector data.
    Vector,
    /// A filter configuration.
    Filter,
    /// Colorize mask information.
    ColorizeMask,
    /// Transformation mask information.
    TransformMask,
    /// Transparency mask information.
    TransparencyMask,
}

//TODO: find what defaultpixel is
//
// Researched from krita/libs/pigment/KoColor.cpp and libs/libkis/Node.cpp
//
// default pixel is of type KoColor
// KoColor contains:
// Metadata: QMap<QString, QVariant>
// m_size: u8
// m_data: [u8; MAX_PIXEL_SIZE]
// m_colorSpace: &KoColorSpace
//
// MAX_PIXEL_SIZE is MAX_CHANNELS_TYPE_SIZE (size of f64) * MAX_CHANNELS_NB (which is 5)
// so is 40 bytes
// m_size is not bigger than max pixel size
//
// defaultpixel by default can be stored in 4 bytes
//
// Questions:
// 1. Do I need to reimplement KoColor at all?
// 2. Do I need Metadata here?
// 3. Can I optimize for space on m_data?
//
// I think having typestate without PhatnomData would be ok choice:
// Color<Colorspace, Unit> {
//   space: Colorspace,
//   unit: Unit,
//   data: [Unit; Colorspace::CHANNELS]
// }
//
// Trait named ChannelCount that contains CHANNELS, and it is implemented for Colorspace variants
// Each variant has to contain unit structs representing individual colorspaces
//
// Look at dasp's impl_frame_for_fixed_size_array!() for inspiration
//
// Decisions:
// 1. I should preserve MAX_PIXEL_SIZE calculation as-is
// 2.
//
// pub type Default

impl Debug for Unloaded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Image => write!(f, "Image"),
            Self::Vector => write!(f, "Vector"),
            Self::Filter => write!(f, "Filter"),
            Self::ColorizeMask => write!(f, "ColorizeMask"),
            Self::TransformMask => write!(f, "TransformMask"),
            Self::TransparencyMask => write!(f, "TransparencyMask"),
            Self::SelectionMask => write!(f, "SelectionMask"),
        }
    }
}

impl Display for Unloaded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Image => write!(f, "raster data"),
            Self::Vector => write!(f, "vector data"),
            Self::Filter => write!(f, "filter configuration"),
            Self::ColorizeMask => write!(f, "colorize mask data"),
            Self::TransformMask => write!(f, "transform mask data"),
            Self::TransparencyMask => write!(f, "transparency mask data"),
            Self::SelectionMask => write!(f, "selection mask data"),
        }
    }
}

impl Debug for Loaded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Image => write!(f, "Image"),
            Self::Vector => write!(f, "Vector"),
            Self::Filter => write!(f, "Filter"),
            Self::ColorizeMask => write!(f, "ColorizeMask"),
            Self::TransformMask => write!(f, "TransformMask"),
            Self::TransparencyMask => write!(f, "TransparencyMask"),
        }
    }
}

impl Display for Loaded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Image => write!(f, "raster data"),
            Self::Vector => write!(f, "vector image data"),
            Self::Filter => write!(f, "filter configuration"),
            Self::ColorizeMask => write!(f, "colorize mask data"),
            Self::TransformMask => write!(f, "transform mask data"),
            Self::TransparencyMask => write!(f, "transparency mask data"),
        }
    }
}

impl fmt::Debug for NodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DoesNotExist => write!(f, "DoesNotExist"),
            Self::Unloaded(inner) => write!(f, "Unloaded({:?})", inner),
            Self::Loaded(inner) => write!(f, "Loaded({:?})", inner),
        }
    }
}

impl fmt::Display for NodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DoesNotExist => write!(f, "non-existent data"),
            Self::Unloaded(inner) => write!(f, "unloaded {}", inner),
            Self::Loaded(inner) => write!(f, "loaded {}", inner),
        }
    }
}
