//! Nodes - layers and masks, and supporting structs.

use std::{
    fmt::{self, Display},
    str::FromStr,
};

use getset::Getters;
use kra_macro::ParseTag;
use quick_xml::events::BytesStart;
use uuid::Uuid;

use crate::{
    error::{MetadataErrorReason, UnknownCompositeOp, XmlError},
    event_get_attr, event_unwrap_as_end, event_unwrap_as_start, next_xml_event, parse_attr,
    parse_bool, parse_layer, Colorspace,
};

/// Composition operator.
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

/// One node (layer or mask) of the image.
#[derive(Debug, Getters)]
#[getset(get = "pub", get_copy = "pub")]
pub struct Node {
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
    //NOTE: masks can't have masks
    masks: Option<Vec<Node>>,
}

impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{0}: {1}", self.uuid, self.name)
    }
}

impl Node {
    pub(crate) fn new(
        common: CommonNodeProps,
        masks: Option<Vec<Node>>,
        node_type: NodeType,
    ) -> Self {
        Node {
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

/// Visibility of a node in the timeline.
#[derive(Debug)]
pub enum InTimeline {
    /// Node is visible in timeline.
    True(Onionskin),
    /// Node is not visible.
    False,
}

/// Whether onionskinning is enabled.
pub type Onionskin = bool;

#[derive(Getters, ParseTag)]
#[getset(get = "pub", get_copy = "pub")]
pub(crate) struct CommonNodeProps {
    #[XmlAttr(pre_parse = "unescape_value()?.into()", fun_override = "name")]
    name: String,
    #[XmlAttr(
        pre_parse = "unescape_value()?",
        fun_override = "Uuid::from_str(uuid.as_ref())?"
    )]
    uuid: Uuid,
    #[XmlAttr(pre_parse = "unescape_value()?.into()", fun_override = "filename")]
    filename: String,
    #[XmlAttr(fun_override = "parse_bool(visible)?")]
    visible: bool,
    #[XmlAttr(fun_override = "parse_bool(locked)?")]
    locked: bool,
    #[XmlAttr(fun_override = "parse_attr(colorlabel)?")]
    colorlabel: u32,
    #[XmlAttr(fun_override = "parse_attr(y)?")]
    y: u32,
    #[XmlAttr(fun_override = "parse_attr(x)?")]
    x: u32,
    #[XmlAttr(
        qname = "intimeline",
        pre_parse = "unescape_value()?",
        fun_override = "parse_in_timeline(in_timeline.as_ref(), tag)?"
    )]
    in_timeline: InTimeline,
}

//parse InTimeline
fn parse_in_timeline(input: &str, tag: &BytesStart) -> Result<InTimeline, MetadataErrorReason> {
    match input {
        "0" => Ok(InTimeline::False),
        "1" => Ok(InTimeline::True(parse_bool(event_get_attr(
            tag,
            "onionskin",
        )?)?)),
        what => {
            return Err(MetadataErrorReason::XmlError(XmlError::ValueError(
                what.to_string(),
            )))
        }
    }
}

/// Types of layers that are recognised.
#[derive(Debug)]
#[non_exhaustive]
pub enum NodeType {
    /// Paint layer.
    PaintLayer(PaintLayerProps),
    /// Group layer, which contains other layers.
    GroupLayer(GroupLayerProps),
    /// Layer that links to a file in the file system.
    FileLayer,
    FilterLayer,
    /// Layer that fills the image with a color.
    FillLayer,
    CloneLayer,
    VectorLayer,
    TransparencyMask,
    FilterMask(FilterMaskProps),
    TransformMask,
    SelectionMask(SelectionMaskProps),
    ColorizeMask,
}

/// Properties specific to paint layer.
#[derive(Debug, Getters, ParseTag)]
#[getset(get = "pub", get_copy = "pub")]
pub struct PaintLayerProps {
    #[XmlAttr(qname = "compositeop", fun_override = "parse_attr(composite_op)?")]
    composite_op: CompositeOp,
    #[XmlAttr(fun_override = "parse_attr(opacity)?")]
    opacity: u8,
    #[XmlAttr(fun_override = "parse_bool(collapsed)?")]
    collapsed: bool,
    #[XmlAttr(
        qname = "colorspacename",
        pre_parse = "unescape_value()?",
        fun_override = "Colorspace::try_from(colorspace.as_ref())?"
    )]
    colorspace: Colorspace,
    #[XmlAttr(
        qname = "channellockflags",
        pre_parse = "unescape_value()?.into_owned()",
        fun_override = "channel_lock_flags"
    )]
    channel_lock_flags: String,
    #[XmlAttr(
        qname = "channelflags",
        pre_parse = "unescape_value()?.into()",
        fun_override = "channel_flags"
    )]
    channel_flags: String,
}

/// Properties specific to group layer.
#[derive(Debug, Getters, ParseTag)]
#[getset(get = "pub", get_copy = "pub")]
#[ExtraArgs(extra_args = "reader: &mut quick_xml::Reader<&[u8]>")]
pub struct GroupLayerProps {
    #[XmlAttr(qname = "compositeop", fun_override = "parse_attr(composite_op)?")]
    pub(crate) composite_op: CompositeOp,
    #[XmlAttr(fun_override = "parse_bool(collapsed)?")]
    pub(crate) collapsed: bool,
    #[XmlAttr(fun_override = "parse_bool(passthrough)?")]
    pub(crate) passthrough: bool,
    #[XmlAttr(fun_override = "parse_attr(opacity)?")]
    pub(crate) opacity: u8,
    #[XmlAttr(extract_data = false, fun_override = "group_get_layers(reader)?")]
    pub(crate) layers: Vec<Node>,
}

// Go over layers in the group, stopping at </layer>
fn group_get_layers(
    reader: &mut quick_xml::Reader<&[u8]>,
) -> Result<Vec<Node>, MetadataErrorReason> {
    let mut layers: Vec<Node> = Vec::new();
    //<layers>
    let event = next_xml_event(reader)?;
    event_unwrap_as_start(event)?;

    loop {
        match parse_layer(reader) {
            Ok(layer) => layers.push(layer),
            Err(MetadataErrorReason::XmlError(XmlError::EventError(a, ref b)))
            // This assumes that we have hit </layers>
                if (a == "layer/mask start event" && b == "layers") =>
            {
                break
            }
            //Actual error
            Err(other) => return Err(other),
        }
    }

    //</layer>
    let event = next_xml_event(reader)?;
    event_unwrap_as_end(event)?;
    Ok(layers)
}

/// Properties specific to filter mask.
#[derive(Debug, Getters, ParseTag)]
#[getset(get = "pub", get_copy = "pub")]
pub struct FilterMaskProps {
    #[XmlAttr(
        qname = "filtername",
        pre_parse = "unescape_value()?",
        fun_override = "filter_name.to_string()"
    )]
    filter_name: String,
    #[XmlAttr(qname = "filterversion", fun_override = "parse_attr(filter_version)?")]
    filter_version: usize,
}

/// Properties specific to selection mask.
#[derive(Debug, Getters, ParseTag)]
#[getset(get = "pub", get_copy = "pub")]
pub struct SelectionMaskProps {
    #[XmlAttr(fun_override = "parse_bool(active)?")]
    active: bool,
}
