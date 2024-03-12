//! Types of layer/mask data.

use core::fmt;

//TODO: store actual data
/// Data that the node refers to via `filename` property.
pub enum NodeData {
    /// Data does not exist (true for clone layers and file layers).
    DoesNotExist,
    /// Data is not loaded (yet).
    NotLoaded,
    /// A compressed image.
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

//TODO: rewrite
impl fmt::Debug for NodeData {
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

impl fmt::Display for NodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DoesNotExist => write!(f, "Non-existent data"),
            Self::NotLoaded => write!(f, "NotLoaded"),
            Self::Image => write!(f, "Image data"),
            Self::Vector => write!(f, "Vector data"),
            Self::Filter => write!(f, "Filter configuration"),
            Self::ColorizeMask => write!(f, "Colorize mask configuration"),
            Self::TransformMask => write!(f, "Transform mask configuration"),
            Self::TransparencyMask => write!(f, "Transparency mask configuration"),
        }
    }
}
