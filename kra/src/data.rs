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

    /// Colorize mask information.
    ColorizeMask,
    /// Transformation mask information.
    TransformMask,
    /// Transparency mask information.
    TransparencyMask,
}

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

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DoesNotExist => write!(f, "DoesNotExist"),
            Self::NotLoaded => write!(f, "NotLoaded"),
            Self::Image => write!(f, "Image"),
            Self::Vector => write!(f, "Vector"),
            Self::Filter => write!(f, "Filter"),
            Self::ColorizeMask => write!(f, "ColorizeMask"),
            Self::TransformMask => write!(f, "TransformMask"),
            Self::TransparencyMask => write!(f, "TransparencyMask"),
        }
    }
}

impl fmt::Debug for NodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DoesNotExist => write!(f, "DoesNotExist"),
            Self::Unloaded(inner) => write!(f, "Unloaded({:?})", inner),
        }
    }
}

impl fmt::Display for NodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DoesNotExist => write!(f, "non-existent data"),
            Self::Unloaded(inner) => write!(f, "unloaded {}", inner),
        }
    }
}
