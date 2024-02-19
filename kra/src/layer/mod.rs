//! Types to define layers.

use std::{
    fmt::{self, Display},
    str::FromStr,
};

use getset::Getters;
use quick_xml::events::BytesStart;

use crate::{
    error::{MetadataErrorReason, UnknownCompositeOp, XmlError},
    event_get_attr,
    metadata::Uuid,
    parse_attr, parse_bool, Colorspace,
};

// TODO: generate functions with a derive macro

#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum CompositeOp {
    Normal,
    Erase,
    In,
    Out,
    AlphaDarken,
    DestinationIn,
    DestinationAtop,
    Xor,
    Or,
    And,
    Nand,
    Nor,
    Xnor,
    Implication,
    NotImplication,
    Converse,
    NotConverse,
    Plus,
    Minus,
    Add,
    Subtract,
    InverseSubtract,
    Diff,
    Multiply,
    Divide,
    ArcTangent,
    GeometricMean,
    AdditiveSubtractive,
    Negation,
    Modulo,
    ModuloContinuous,
    DivisiveModulo,
    DivisiveModuloContinuous,
    ModuloShift,
    ModuloShiftContinuous,
    Equivalence,
    Allanon,
    Parallel,
    GrainMerge,
    GrainExtract,
    Exclusion,
    HardMix,
    HardMixPhotoshop,
    HardMixSofterPhotoshop,
    Overlay,
    Behind,
    Greater,
    HardOverlay,
    Interpolation,
    Interpolation2X,
    PenumbraA,
    PenumbraB,
    PenumbraC,
    PenumbraD,
    Darken,
    Burn,
    LinearBurn,
    GammaDark,
    ShadeIfsIllusions,
    FogDarkenIfsIllusions,
    EasyBurn,
    Lighten,
    Dodge,
    LinearDodge,
    Screen,
    HardLight,
    SoftLightIfsIllusions,
    SoftLightPegtopDelphi,
    SoftLight,
    SoftLightSvg,
    GammaLight,
    GammaIllumination,
    VividLight,
    FlatLight,
    LinearLight,
    PinLight,
    PnormA,
    PnormB,
    SuperLight,
    TintIfsIllusions,
    FogLightenIfsIllusions,
    EasyDodge,
    LuminositySai,
    Hue,
    Color,
    Saturation,
    IncSaturation,
    DecSaturation,
    Luminize,
    IncLuminosity,
    DecLuminosity,
    HueHsv,
    ColorHsv,
    SaturationHsv,
    IncSaturationHsv,
    DecSaturationHsv,
    Value,
    IncValue,
    DecValue,
    HueHsl,
    ColorHsl,
    SaturationHsl,
    IncSaturationHsl,
    DecSaturationHsl,
    Lightness,
    IncLightness,
    DecLightness,
    HueHsi,
    ColorHsi,
    SaturationHsi,
    IncSaturationHsi,
    DecSaturationHsi,
    Intensity,
    IncIntensity,
    DecIntensity,
    Copy,
    CopyRed,
    CopyGreen,
    CopyBlue,
    TangentNormalmap,
    Colorize,
    Bumpmap,
    CombineNormal,
    Clear,
    Dissolve,
    Displace,
    Nocomposition,
    PassThrough, //from source: not implemented anywhere yet
    DarkerColor,
    LighterColor,
    Undefined,
    Reflect,
    Glow,
    Freeze,
    Heat,
    GlowHeat,
    HeatGlow,
    ReflectFreeze,
    FreezeReflect,
    HeatGlowFreezeReflectHybrid,
    LambertLighting,
    LambertLightingGamma22,
}

impl FromStr for CompositeOp {
    type Err = UnknownCompositeOp;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "normal" => CompositeOp::Normal,
            "erase" => CompositeOp::Erase,
            "in" => CompositeOp::In,
            "out" => CompositeOp::Out,
            "alphadarken" => CompositeOp::AlphaDarken,
            "destination-in" => CompositeOp::DestinationIn,
            "destination-atop" => CompositeOp::DestinationAtop,
            "xor" => CompositeOp::Xor,
            "or" => CompositeOp::Or,
            "and" => CompositeOp::And,
            "nand" => CompositeOp::Nand,
            "nor" => CompositeOp::Nor,
            "xnor" => CompositeOp::Xnor,
            "implication" => CompositeOp::Implication,
            "not_implication" => CompositeOp::NotImplication,
            "converse" => CompositeOp::Converse,
            "not_converse" => CompositeOp::NotConverse,
            "plus" => CompositeOp::Plus,
            "minus" => CompositeOp::Minus,
            "add" => CompositeOp::Add,
            "subtract" => CompositeOp::Subtract,
            "inverse_subtract" => CompositeOp::InverseSubtract,
            "diff" => CompositeOp::Diff,
            "multiply" => CompositeOp::Multiply,
            "divide" => CompositeOp::Divide,
            "arc_tangent" => CompositeOp::ArcTangent,
            "geometric_mean" => CompositeOp::GeometricMean,
            "additive_subtractive" => CompositeOp::AdditiveSubtractive,
            "negation" => CompositeOp::Negation,
            "modulo" => CompositeOp::Modulo,
            "modulo_continuous" => CompositeOp::ModuloContinuous,
            "divisive_modulo" => CompositeOp::DivisiveModulo,
            "divisive_modulo_continuous" => CompositeOp::DivisiveModuloContinuous,
            "modulo_shift" => CompositeOp::ModuloShift,
            "modulo_shift_continuous" => CompositeOp::ModuloShiftContinuous,
            "equivalence" => CompositeOp::Equivalence,
            "allanon" => CompositeOp::Allanon,
            "parallel" => CompositeOp::Parallel,
            "grain_merge" => CompositeOp::GrainMerge,
            "grain_extract" => CompositeOp::GrainExtract,
            "exclusion" => CompositeOp::Exclusion,
            "hard mix" => CompositeOp::HardMix,
            "hard_mix_photoshop" => CompositeOp::HardMixPhotoshop,
            "hard_mix_softer_photoshop" => CompositeOp::HardMixSofterPhotoshop,
            "overlay" => CompositeOp::Overlay,
            "behind" => CompositeOp::Behind,
            "greater" => CompositeOp::Greater,
            "hard overlay" => CompositeOp::HardOverlay,
            "interpolation" => CompositeOp::Interpolation,
            "interpolation 2x" => CompositeOp::Interpolation2X,
            "penumbra a" => CompositeOp::PenumbraA,
            "penumbra b" => CompositeOp::PenumbraB,
            "penumbra c" => CompositeOp::PenumbraC,
            "penumbra d" => CompositeOp::PenumbraD,
            "darken" => CompositeOp::Darken,
            "burn" => CompositeOp::Burn,
            "linear_burn" => CompositeOp::LinearBurn,
            "gamma_dark" => CompositeOp::GammaDark,
            "shade_ifs_illusions" => CompositeOp::ShadeIfsIllusions,
            "fog_darken_ifs_illusions" => CompositeOp::FogDarkenIfsIllusions,
            "easy burn" => CompositeOp::EasyBurn,
            "lighten" => CompositeOp::Lighten,
            "dodge" => CompositeOp::Dodge,
            "linear_dodge" => CompositeOp::LinearDodge,
            "screen" => CompositeOp::Screen,
            "hard_light" => CompositeOp::HardLight,
            "soft_light_ifs_illusions" => CompositeOp::SoftLightIfsIllusions,
            "soft_light_pegtop_delphi" => CompositeOp::SoftLightPegtopDelphi,
            "soft_light" => CompositeOp::SoftLight,
            "soft_light_svg" => CompositeOp::SoftLightSvg,
            "gamma_light" => CompositeOp::GammaLight,
            "gamma_illumination" => CompositeOp::GammaIllumination,
            "vivid_light" => CompositeOp::VividLight,
            "flat_light" => CompositeOp::FlatLight,
            "linear light" => CompositeOp::LinearLight,
            "pin_light" => CompositeOp::PinLight,
            "pnorm_a" => CompositeOp::PnormA,
            "pnorm_b" => CompositeOp::PnormB,
            "super_light" => CompositeOp::SuperLight,
            "tint_ifs_illusions" => CompositeOp::TintIfsIllusions,
            "fog_lighten_ifs_illusions" => CompositeOp::FogLightenIfsIllusions,
            "easy dodge" => CompositeOp::EasyDodge,
            "luminosity_sai" => CompositeOp::LuminositySai,
            "hue" => CompositeOp::Hue,
            "color" => CompositeOp::Color,
            "saturation" => CompositeOp::Saturation,
            "inc_saturation" => CompositeOp::IncSaturation,
            "dec_saturation" => CompositeOp::DecSaturation,
            "luminize" => CompositeOp::Luminize,
            "inc_luminosity" => CompositeOp::IncLuminosity,
            "dec_luminosity" => CompositeOp::DecLuminosity,
            "hue_hsv" => CompositeOp::HueHsv,
            "color_hsv" => CompositeOp::ColorHsv,
            "saturation_hsv" => CompositeOp::SaturationHsv,
            "inc_saturation_hsv" => CompositeOp::IncSaturationHsv,
            "dec_saturation_hsv" => CompositeOp::DecSaturationHsv,
            "value" => CompositeOp::Value,
            "inc_value" => CompositeOp::IncValue,
            "dec_value" => CompositeOp::DecValue,
            "hue_hsl" => CompositeOp::HueHsl,
            "color_hsl" => CompositeOp::ColorHsl,
            "saturation_hsl" => CompositeOp::SaturationHsl,
            "inc_saturation_hsl" => CompositeOp::IncSaturationHsl,
            "dec_saturation_hsl" => CompositeOp::DecSaturationHsl,
            "lightness" => CompositeOp::Lightness,
            "inc_lightness" => CompositeOp::IncLightness,
            "dec_lightness" => CompositeOp::DecLightness,
            "hue_hsi" => CompositeOp::HueHsi,
            "color_hsi" => CompositeOp::ColorHsi,
            "saturation_hsi" => CompositeOp::SaturationHsi,
            "inc_saturation_hsi" => CompositeOp::IncSaturationHsi,
            "dec_saturation_hsi" => CompositeOp::DecSaturationHsi,
            "intensity" => CompositeOp::Intensity,
            "inc_intensity" => CompositeOp::IncIntensity,
            "dec_intensity" => CompositeOp::DecIntensity,
            "copy" => CompositeOp::Copy,
            "copy_red" => CompositeOp::CopyRed,
            "copy_green" => CompositeOp::CopyGreen,
            "copy_blue" => CompositeOp::CopyBlue,
            "tangent_normalmap" => CompositeOp::TangentNormalmap,
            "colorize" => CompositeOp::Colorize,
            "bumpmap" => CompositeOp::Bumpmap,
            "combine_normal" => CompositeOp::CombineNormal,
            "clear" => CompositeOp::Clear,
            "dissolve" => CompositeOp::Dissolve,
            "displace" => CompositeOp::Displace,
            "nocomposition" => CompositeOp::Nocomposition,
            "pass through" => CompositeOp::PassThrough,
            "darker color" => CompositeOp::DarkerColor,
            "lighter color" => CompositeOp::LighterColor,
            "undefined" => CompositeOp::Undefined,
            "reflect" => CompositeOp::Reflect,
            "glow" => CompositeOp::Glow,
            "freeze" => CompositeOp::Freeze,
            "heat" => CompositeOp::Heat,
            "glow_heat" => CompositeOp::GlowHeat,
            "heat_glow" => CompositeOp::HeatGlow,
            "reflect_freeze" => CompositeOp::ReflectFreeze,
            "freeze_reflect" => CompositeOp::FreezeReflect,
            "heat_glow_freeze_reflect_hybrid" => CompositeOp::HeatGlowFreezeReflectHybrid,
            "lambert_lighting" => CompositeOp::LambertLighting,
            "lambert_lighting_gamma2.2" => CompositeOp::LambertLightingGamma22,
            _ => return Err(UnknownCompositeOp(s.to_owned())),
        })
    }
}

#[derive(Debug, Getters)]
#[getset(get = "pub", get_copy = "pub")]
pub struct Node {
    props: NodeProps,
    image: Option<Vec<u8>>,
}

impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{0}: {1}", self.props.uuid, self.props.name)
    }
}

impl Node {
    pub(crate) fn new(props: NodeProps, image: Option<Vec<u8>>) -> Self {
        Node { props, image }
    }
}

// properties common to all nodes + properties that are node-specific
#[derive(Debug, Getters)]
#[getset(get = "pub", get_copy = "pub")]
pub struct NodeProps {
    name: String,
    uuid: Uuid,
    filename: String,
    visible: bool,
    locked: bool,
    colorlabel: u32,
    node_type: NodeType,
    y: u32,
    x: u32,
    in_timeline: InTimeline,
    //TODO: masks can't have masks
    // Because it implies that the mask's event is
    // always closed, the field will always be None.
    // Still, should consider moving it out.
    masks: Option<Vec<Node>>,
}

impl NodeProps {
    pub(crate) fn from_parts(
        common: CommonNodeProps,
        masks: Option<Vec<Node>>,
        node_type: NodeType,
    ) -> Self {
        NodeProps {
            name: common.name,
            uuid: common.uuid,
            filename: common.filename,
            visible: common.visible,
            locked: common.locked,
            colorlabel: common.colorlabel,
            node_type,
            y: common.y,
            x: common.x,
            in_timeline: common.in_timeline,
            masks,
        }
    }
}

#[derive(Debug)]
pub enum InTimeline {
    True(Onionskin),
    False,
}

pub type Onionskin = bool;

#[derive(Getters)]
#[getset(get = "pub", get_copy = "pub")]
pub(crate) struct CommonNodeProps {
    name: String,
    uuid: Uuid,
    filename: String,
    visible: bool,
    locked: bool,
    colorlabel: u32,
    y: u32,
    x: u32,
    in_timeline: InTimeline,
}

impl CommonNodeProps {
    pub(crate) fn parse_tag(tag: &BytesStart) -> Result<Self, MetadataErrorReason> {
        let name = event_get_attr(tag, "name")?.unescape_value()?.into();
        let uuid = event_get_attr(tag, "uuid")?.unescape_value()?;
        let filename = event_get_attr(tag, "filename")?.unescape_value()?.into();
        let visible = event_get_attr(tag, "visible")?;
        let locked = event_get_attr(tag, "locked")?;
        let colorlabel = event_get_attr(tag, "colorlabel")?;
        let x = event_get_attr(tag, "x")?;
        let y = event_get_attr(tag, "y")?;

        let in_timeline = match event_get_attr(tag, "intimeline")?
            .unescape_value()?
            .as_ref()
        {
            "0" => InTimeline::False,
            "1" => InTimeline::True(parse_bool(event_get_attr(tag, "onionskin")?)?),
            what => {
                return Err(MetadataErrorReason::XmlError(XmlError::ValueError(
                    what.to_string(),
                )))
            }
        };

        Ok(CommonNodeProps {
            name,
            uuid: Uuid::from_str(uuid.as_ref())?,
            filename,
            visible: parse_bool(visible)?,
            locked: parse_bool(locked)?,
            colorlabel: parse_attr(colorlabel)?,
            y: parse_attr(y)?,
            x: parse_attr(x)?,
            in_timeline,
        })
    }
}

/// Types of layers that are recognised.
#[derive(Debug)]
#[non_exhaustive]
pub enum NodeType {
    PaintLayer(PaintLayerProps),
    GroupLayer(GroupLayerProps),
    FileLayer,
    FilterLayer,
    FillLayer,
    CloneLayer,
    VectorLayer,
    TransparencyMask,
    FilterMask(FilterMaskProps),
    TransformMask,
    SelectionMask(SelectionMaskProps),
    ColorizeMask,
}

/// Paint layer.
#[derive(Debug, Getters)]
#[getset(get = "pub", get_copy = "pub")]
pub struct PaintLayerProps {
    composite_op: CompositeOp,
    opacity: u8,
    collapsed: bool,
    colorspace: Colorspace,
    channel_lock_flags: String,
    channel_flags: String,
}

impl PaintLayerProps {
    pub(crate) fn parse_tag(tag: &BytesStart) -> Result<Self, MetadataErrorReason> {
        let composite_op = event_get_attr(tag, "compositeop")?;
        let collapsed = event_get_attr(tag, "collapsed")?;
        let opacity = event_get_attr(tag, "opacity")?;
        let colorspace = event_get_attr(tag, "colorspacename")?.unescape_value()?;
        let channel_lock_flags = event_get_attr(tag, "channellockflags")?
            .unescape_value()?
            .into_owned();
        let channel_flags = event_get_attr(tag, "channelflags")?.unescape_value()?;
        Ok(PaintLayerProps {
            composite_op: parse_attr(composite_op)?,
            opacity: parse_attr(opacity)?,
            collapsed: parse_bool(collapsed)?,
            colorspace: Colorspace::try_from(colorspace.as_ref())?,
            channel_lock_flags,
            channel_flags: channel_flags.into(),
        })
    }
}

/// Group layer.
#[derive(Debug, Getters)]
#[getset(get = "pub", get_copy = "pub")]
pub struct GroupLayerProps {
    pub(crate) composite_op: CompositeOp,
    pub(crate) collapsed: bool,
    pub(crate) passthrough: bool,
    pub(crate) opacity: u8,
    pub(crate) layers: Vec<Node>,
}
// Group layers are more complex because they can have layers.
// This requires either splitting the layers away, which would make structure confusing,
// or giving reader and parse_layer() to the function, which, too, would be confusing.
// So I will not make a parse_tag() for group layer props.

/// Filter mask.
#[derive(Debug, Getters)]
#[getset(get = "pub", get_copy = "pub")]
pub struct FilterMaskProps {
    filter_name: String,
    filter_version: usize,
}

impl FilterMaskProps {
    pub(crate) fn parse_tag(tag: &BytesStart) -> Result<Self, MetadataErrorReason> {
        let filter_version = event_get_attr(tag, "filterversion")?;
        let filter_name = event_get_attr(tag, "filtername")?.unescape_value()?;
        Ok(FilterMaskProps {
            filter_name: filter_name.to_string(),
            filter_version: parse_attr(filter_version)?,
        })
    }
}

/// Filter mask.
#[derive(Debug, Getters)]
#[getset(get = "pub", get_copy = "pub")]
pub struct SelectionMaskProps {
    active: bool,
}

impl SelectionMaskProps {
    pub(crate) fn parse_tag(tag: &BytesStart) -> Result<Self, MetadataErrorReason> {
        let active = event_get_attr(tag, "active")?;
        Ok(SelectionMaskProps {
            active: parse_bool(active)?,
        })
    }
}
