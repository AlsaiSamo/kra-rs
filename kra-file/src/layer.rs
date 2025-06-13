//! Nodes - layers and masks.

use std::path::{Path, PathBuf};
use std::str::FromStr;

use enum_as_inner::EnumAsInner;
use getset::Setters;
use kra_macro::ParseTag;
use quick_xml::events::BytesStart;
use uuid::Uuid;

#[cfg(not(feature = "data"))]
use crate::dummy::{Colorspace, CompositeOp};
use crate::error::{MetadataErrorReason, XmlError};
use crate::helper::{event_get_attr, parse_attr, parse_bool};

// TODO: move the types to a separate module.
// Later, when creating the types crate, move them there.

// TODO: restructure the project:
// 1. Enforce "data" feature (by erroring on compiling with it).

// TODO: setters and mut getters from getset

macro_rules! getter_func {
    ($vis:vis $prop:ident -> &str) => {
        #[doc = concat!("Return reference to inner field `", stringify!($prop), "`")]
        $vis fn $prop(&self) -> &str {
            self.$prop.as_str()
        }
    };
    ($vis:vis $prop:ident -> &Path) => {
        #[doc = concat!("Return reference to inner field `", stringify!($prop), "`")]
        $vis fn $prop(&self) -> &Path {
            self.$prop.as_path()
        }
    };
    ($vis:vis $prop:ident -> &[Node]) => {
        #[doc = concat!("Return reference to inner field `", stringify!($prop), "`")]
        $vis fn $prop(&self) -> &[Node] {
            self.$prop.as_slice()
        }
    };
    ($vis:vis $prop:ident -> &Uuid) => {
        #[doc = concat!("Return reference to inner field `", stringify!($prop), "`")]
        $vis fn $prop(&self) -> &Uuid {
            &self.$prop
        }
    };
    ($vis:vis $prop:ident -> $type:ty) => {
        #[doc = concat!("Return inner field `", stringify!($prop), "`")]
        $vis fn $prop(&self) -> $type {
            self.$prop.into()
        }
    };
}

/// A node, which is either a layer or a mask.
#[derive(Debug, EnumAsInner, Clone)]
pub enum Node {
    /// Paint layer.
    /// Include
    PaintLayer(PaintLayer),
    /// Group layer.
    /// It is a composite of other layers.
    GroupLayer(GroupLayer),
    /// File layer.
    /// Takes an image from a file external to `.kra` file.
    FileLayer(FileLayer),
    /// Filter layer, also called "adjustmentlayer" in the metadata.
    FilterLayer(FilterLayer),
    /// Fill layer, also called "generatorlayer" in the metadata.
    /// This fills the layer with a solid color.
    FillLayer(FillLayer),
    /// Layer that clones another layer.
    CloneLayer(CloneLayer),
    /// Vector layer, also called "shapelayer" in metadata.
    VectorLayer(VectorLayer),
    /// Transparency mask.
    TransparencyMask(TransparencyMask),
    /// Filter mask.
    FilterMask(FilterMask),
    /// Transform mask.
    TransformMask(TransformMask),
    /// Selection mask.
    SelectionMask(SelectionMask),
    /// Colorize mask.
    ColorizeMask(ColorizeMask),
}

// NOTE: $$ not stabilised :(
// Forward getters from Node enum to the inner type if it is possible
macro_rules! node_enum_getter {
    ($funcname:ident -> $returntype:ty, [$($item:ident),*]) => {
    #[doc = concat!("Return reference to inner field `", stringify!($funcname), "` if the node of this type contains it")]
    pub fn $funcname(&self) -> Option<$returntype> {
        match self {
            $(Node::$item(node) => {Some(node.$funcname())},)*
            #[allow(unreachable_patterns)]
            _ => None
        }
    }
    };
}

// NOTE: the funcname should follow getset's naming scheme
macro_rules! node_enum_setter {
    ($funcname:ident($input:ty), [$($item:ident),*]) => {
    #[doc = concat!("Set inner field `", stringify!($funcname), "` from `", stringify!($input),"` if the node of this type has this field")]
    pub fn $funcname(&mut self, input: $input) -> Result<(),()> {
        match self {
            $(Node::$item(node) => {
                // Some(node.$funcname())
                node.$funcname(input);
                Ok(())
            },)*
            #[allow(unreachable_patterns)]
            _ => Err(())
        }
    }
    };
}

impl Node {
    /// Is the node a layer.
    /// If it is, properties from `LayerProperties` can be used.
    pub fn is_layer(&self) -> bool {
        match self {
            Node::PaintLayer(_) => true,
            Node::GroupLayer(_) => true,
            Node::FileLayer(_) => true,
            Node::FilterLayer(_) => true,
            Node::FillLayer(_) => true,
            Node::CloneLayer(_) => true,
            Node::VectorLayer(_) => true,
            _ => false,
        }
    }

    /// Is the node a mask.
    pub fn is_mask(&self) -> bool {
        match self {
            Node::TransparencyMask(_) => true,
            Node::FilterMask(_) => true,
            Node::TransformMask(_) => true,
            Node::SelectionMask(_) => true,
            Node::ColorizeMask(_) => true,
            _ => false,
        }
    }

    /// Is the node a layer.
    /// If it is, properties from `FilterProperties` can be used.
    pub fn is_filter(&self) -> bool {
        match self {
            Node::FilterLayer(_) => true,
            Node::FilterMask(_) => true,
            _ => false,
        }
    }

    /// Does the node have a composition operation specified.
    /// If it does, it can be accessed through `CompositeOpProperty` trait.
    pub fn has_composite_op(&self) -> bool {
        match self {
            Node::PaintLayer(_) => true,
            Node::FilterLayer(_) => true,
            Node::FillLayer(_) => true,
            Node::FileLayer(_) => true,
            Node::CloneLayer(_) => true,
            Node::ColorizeMask(_) => true,
            Node::VectorLayer(_) => true,
            Node::GroupLayer(_) => true,
            _ => false,
        }
    }

    /// Can the layer be painted on.
    /// If true, then it has channel flags and masks, accessible through `PaintableLayerProperties` trait.
    pub fn is_paintable_layer(&self) -> bool {
        match self {
            Node::PaintLayer(_) => true,
            Node::FileLayer(_) => true,
            Node::FilterLayer(_) => true,
            Node::FillLayer(_) => true,
            Node::CloneLayer(_) => true,
            Node::VectorLayer(_) => true,
            _ => false,
        }
    }

    /// Does the node specify a colorspace.
    /// If it does, it is accessible through `ColorspaceProperty` trait.
    pub fn has_colorspace(&self) -> bool {
        match self {
            Node::PaintLayer(_) => true,
            Node::FileLayer(_) => true,
            Node::ColorizeMask(_) => true,
            _ => false,
        }
    }
    // TODO: common node props should not be behind Option
    node_enum_getter!(name -> &str, [
        PaintLayer, FileLayer, FilterLayer, FillLayer, CloneLayer, ColorizeMask,
        VectorLayer, GroupLayer, FilterMask, SelectionMask, TransparencyMask, TransformMask
    ]);
    node_enum_getter!(uuid -> &Uuid, [
        PaintLayer, FileLayer, FilterLayer, FillLayer, CloneLayer, ColorizeMask,
        VectorLayer, GroupLayer, FilterMask, SelectionMask, TransparencyMask, TransformMask
    ]);
    node_enum_getter!(filename -> &str, [
        PaintLayer, FileLayer, FilterLayer, FillLayer, CloneLayer, ColorizeMask,
        VectorLayer, GroupLayer, FilterMask, SelectionMask, TransparencyMask, TransformMask
    ]);
    node_enum_getter!(visible -> bool, [
        PaintLayer, FileLayer, FilterLayer, FillLayer, CloneLayer, ColorizeMask,
        VectorLayer, GroupLayer, FilterMask, SelectionMask, TransparencyMask, TransformMask
    ]);
    node_enum_getter!(locked -> bool, [
        PaintLayer, FileLayer, FilterLayer, FillLayer, CloneLayer, ColorizeMask,
        VectorLayer, GroupLayer, FilterMask, SelectionMask, TransparencyMask, TransformMask
    ]);
    node_enum_getter!(colorlabel -> u32, [
        PaintLayer, FileLayer, FilterLayer, FillLayer, CloneLayer, ColorizeMask,
        VectorLayer, GroupLayer, FilterMask, SelectionMask, TransparencyMask, TransformMask
    ]);
    node_enum_getter!(y -> i32, [
        PaintLayer, FileLayer, FilterLayer, FillLayer, CloneLayer, ColorizeMask,
        VectorLayer, GroupLayer, FilterMask, SelectionMask, TransparencyMask, TransformMask
    ]);
    node_enum_getter!(x -> i32, [
        PaintLayer, FileLayer, FilterLayer, FillLayer, CloneLayer, ColorizeMask,
        VectorLayer, GroupLayer, FilterMask, SelectionMask, TransparencyMask, TransformMask
    ]);
    node_enum_getter!(in_timeline -> InTimeline, [
        PaintLayer, FileLayer, FilterLayer, FillLayer, CloneLayer, ColorizeMask,
        VectorLayer, GroupLayer, FilterMask, SelectionMask, TransparencyMask, TransformMask
    ]);
    node_enum_getter!(composite_op -> CompositeOp, [
        PaintLayer, FileLayer, FilterLayer, FillLayer, CloneLayer, ColorizeMask, VectorLayer, GroupLayer
    ]);
    node_enum_getter!(collapsed -> bool, [
        PaintLayer, FileLayer, FilterLayer, FillLayer, CloneLayer, VectorLayer, GroupLayer
    ]);
    node_enum_getter!(opacity -> u8, [
        PaintLayer, FileLayer, FilterLayer, FillLayer, CloneLayer, VectorLayer, GroupLayer
    ]);
    node_enum_getter!(channel_flags -> &str, [
        PaintLayer, FileLayer, FilterLayer, FillLayer, CloneLayer, VectorLayer
    ]);
    node_enum_getter!(masks -> &[Node], [
        PaintLayer, FileLayer, FilterLayer, FillLayer, CloneLayer, VectorLayer
    ]);
    node_enum_getter!(colorspace -> Colorspace, [
        PaintLayer, FileLayer, ColorizeMask
    ]);
    node_enum_getter!(filter_name -> &str, [
        FilterLayer, FilterMask
    ]);
    node_enum_getter!(filter_version -> u32, [
        FilterLayer, FilterMask
    ]);

    node_enum_setter!(
        set_masks(Vec<Node>),
        [
            PaintLayer,
            FileLayer,
            FilterLayer,
            FillLayer,
            CloneLayer,
            VectorLayer
        ]
    );
}

// TODO: proper docs for functions
// NOTE: due to dollar-dollar not being stabilised I cannot write nested
// repetition clenaly.
// More here: https://github.com/rust-lang/rust/issues/35853
// Too bad... it could have been much neater.
//
// Creates the getter trait and implement it for the given structs
macro_rules! make_getters_trait {
    // traitname is the name of the trait that is created
    // prop is the name of a field for which we generate the getter
    // type is its type
    // structname is the name of the struct for which we implement the trait
    ($(#[$traitmeta:meta])* $traitname:ident, {
         $prop1:ident -> $type1:ty,
         $prop2:ident -> $type2:ty,
         $prop3:ident -> $type3:ty,
         $prop4:ident -> $type4:ty,
         $prop5:ident -> $type5:ty,
         $prop6:ident -> $type6:ty,
         $prop7:ident -> $type7:ty,
         $prop8:ident -> $type8:ty,
         $prop9:ident -> $type9:ty
     },
     [
         $($structname:ident),*
     ]) =>
        (paste::item! {::defile::defile! {
        $(#[$traitmeta])*
        pub trait $traitname {
        #[doc = concat!("`", stringify!($prop1), "`")]
            fn $prop1(&self) -> $type1;
        #[doc = concat!("`", stringify!($prop2), "`")]
            fn $prop2(&self) -> $type2;
        #[doc = concat!("`", stringify!($prop3), "`")]
            fn $prop3(&self) -> $type3;
        #[doc = concat!("`", stringify!($prop4), "`")]
            fn $prop4(&self) -> $type4;
        #[doc = concat!("`", stringify!($prop5), "`")]
            fn $prop5(&self) -> $type5;
        #[doc = concat!("`", stringify!($prop6), "`")]
            fn $prop6(&self) -> $type6;
        #[doc = concat!("`", stringify!($prop7), "`")]
            fn $prop7(&self) -> $type7;
        #[doc = concat!("`", stringify!($prop8), "`")]
            fn $prop8(&self) -> $type8;
        #[doc = concat!("`", stringify!($prop9), "`")]
            fn $prop9(&self) -> $type9;
        }

        $(impl $traitname for $structname {
                getter_func!{$prop1 -> @$type1}
                getter_func!{$prop2 -> @$type2}
                getter_func!{$prop3 -> @$type3}
                getter_func!{$prop4 -> @$type4}
                getter_func!{$prop5 -> @$type5}
                getter_func!{$prop6 -> @$type6}
                getter_func!{$prop7 -> @$type7}
                getter_func!{$prop8 -> @$type8}
                getter_func!{$prop9 -> @$type9}
        })*
            }});

    ($(#[$traitmeta:meta])* $traitname:ident, {
         $prop1:ident -> $type1:ty,
         $prop2:ident -> $type2:ty
     },
     [
         $($structname:ident),*
     ]) =>
        (paste::item! {::defile::defile!
         {
        $(#[$traitmeta])*
        pub trait $traitname: CommonNodeProperties
            {
        #[doc = concat!("`", stringify!($prop1), "`")]
            fn $prop1(&self) -> $type1;
        #[doc = concat!("`", stringify!($prop2), "`")]
            fn $prop2(&self) -> $type2;
            }

        $(impl $traitname for $structname {
                getter_func!{$prop1 -> @$type1}
                getter_func!{$prop2 -> @$type2}
        })*
         }});

    ($(#[$traitmeta:meta])* $traitname:ident, {
         $prop1:ident -> $type1:ty
     },
     [
         $($structname:ident),*
     ]) =>
        (paste::item! {::defile::defile! {
        $(#[$traitmeta])*
        pub trait $traitname: CommonNodeProperties {
        #[doc = concat!("`", stringify!($prop1), "`")]
            fn $prop1(&self) -> $type1;
        }

        $(impl $traitname for $structname {
                getter_func!{$prop1 -> @$type1}
        })*
        }})
    // NOTE: the code above is the unrolled version of this
    // If #![feature(macro_metavar_expr)] gets stabilised,
    // or at least `$$` syntax does,
    // celebrate the occasion and use this code instead of the unrolled one.
    // ($traitname:ident: $($supertrait:ident)?,
    //  {
    //      $($prop:ident -> $type:ty),*
    //  },
    //  [
    //      $($structname:ident),*
    //  ]) =>
    //     (::defile::defile! {
    //     //pub trait $traitname$(:$supertrait)? {
    //     pub trait $traitname {
    //         $(fn $prop(&self) -> $type ;)*
    //     }

    //     $(impl $traitname for $structname {
    //         $(
    //             getter_func!{$prop -> @$type}
    //         )*
    //     })*
    // })
}

make_getters_trait! {
    #[doc = "Properties of a filter (`FilterLayer` or `FilterMask`)."]
    FilterProperties,
    {
        filter_name -> &str,
        filter_version -> u32
    },
    [
        FilterLayer,
        FilterMask
    ]
}

make_getters_trait! {
    #[doc = "Access to properties of layers (not masks)."]
    LayerProperties,
    {
        collapsed -> bool,
        opacity -> u8
    },
    [
        PaintLayer,
        FileLayer,
        FilterLayer,
        FillLayer,
        CloneLayer,
        VectorLayer,
        GroupLayer
    ]
}

make_getters_trait! {
    #[doc = "Access to properties of layers that can be painted on
    (not group layer and not masks)."]
    PaintableLayerProperties,
    {
        channel_flags -> &str,
        masks -> &[Node]
    },
    [
        PaintLayer,
        FileLayer,
        FilterLayer,
        FillLayer,
        CloneLayer,
        VectorLayer
    ]
}

make_getters_trait! {
    #[doc = "Access to `colorspace` property of `PaintLayer`, `FileLayer` and `ColorizeMask`."]
    ColorspaceProperty,
    {
        colorspace -> Colorspace
    },
    [
        PaintLayer,
        FileLayer,
        ColorizeMask
    ]
}

make_getters_trait! {
    #[doc = "Access to `composite_op` property of layers and `ColorizeMask`."]
    CompositeOpProperty,
    {
        composite_op -> CompositeOp
    },
    [
        PaintLayer,
        FileLayer,
        FilterLayer,
        FillLayer,
        CloneLayer,
        ColorizeMask,
        VectorLayer,
        GroupLayer
    ]
}

make_getters_trait! {
    #[doc = "Access to properties common to every type of node."]
    CommonNodeProperties,
    {
        name -> &str,
        uuid -> &Uuid,
        filename -> &str,
        visible -> bool,
        locked -> bool,
        colorlabel -> u32,
        y -> i32,
        x -> i32,
        in_timeline -> InTimeline
    },
    [
        PaintLayer,
        FileLayer,
        FilterLayer,
        FillLayer,
        CloneLayer,
        VectorLayer,
        GroupLayer,
        FilterMask,
        SelectionMask,
        TransparencyMask,
        TransformMask,
        ColorizeMask
    ]
}

// NOTE: hygiene issues complicate generation of $propsname structs
// if I can resolve this, I'll be able to generate props right here in the macro
// and get extra style points
macro_rules! make_node {
    (
        // struct's meta
        $(#[$structmeta:meta])*
        // name of the struct
        $name:ident,
        // name of the property struct (that is currently provided externally)
        $propsname:ident,
        {
            // propsmeta is written in #+[] - currently unused, was meant to provide
            // meta to fields in property struct
            // fieldmeta is written in #[] - provides meta to fields in the struct
            $($(#+[$propsmeta:meta])* $(#[$fieldmeta:meta])* $field:ident:$type:ty),*
        }
    ) => {
        #[derive(Debug, Clone, Setters)]
        #[getset(set)]
        $(#[$structmeta])*
        pub struct $name {
            // Common node props
            name: String,
            uuid: Uuid,
            filename: String,
            visible: bool,
            locked: bool,
            colorlabel: u32,
            y: i32,
            x: i32,
            in_timeline: InTimeline,
            // Unique node props
            $($(#[$fieldmeta])?
                $field: $type,)*
        }

        // #[derive(Debug, ParseTag)]
        // pub(crate) struct $propsname {
        //     $($(#[$propsmeta])?
        //       $field:$type,)*
        // }

        impl $name {
            #[allow(unused_variables)]
            pub(crate) fn new (common: CommonNodeProps, unique: $propsname) -> Self {
                $name{
                    name: common.name,
                    uuid: common.uuid,
                    filename: common.filename,
                    visible: common.visible,
                    locked: common.locked,
                    colorlabel: common.colorlabel,
                    y: common.y,
                    x: common.x,
                    in_timeline: common.in_timeline,
                    $($field: unique.$field,)*
                }
            }
        }
    };
}

make_node!(
    #[doc = "Filter mask."]
    FilterMask,
    FilterMaskProps,
    {
        filter_name: String,
        filter_version: u32
    }
);

make_node!(
    #[doc = "Paint layer."]
    PaintLayer,
    PaintLayerProps,
    {
    channel_flags: String,
    channel_lock_flags: String,
    colorspace: Colorspace,
    collapsed: bool,
    opacity: u8,
    composite_op: CompositeOp,
    masks: Vec<Node>
    }
);

impl PaintLayer {
    getter_func!(pub channel_lock_flags -> &str);
}

make_node!(
    #[doc = "Selection mask."]
    SelectionMask,
    SelectionMaskProps,
    {
        active: bool
    }
);

impl SelectionMask {
    getter_func!(pub active -> bool);
}

make_node!(
    #[doc = "File layer."]
    FileLayer,
    FileLayerProps,
    {
        collapsed: bool,
        scaling_filter: String,
        scale: bool,
        composite_op: CompositeOp,
        opacity: u8,
        colorspace: Colorspace,
        scaling_method: u32,
        source: PathBuf,
        channel_flags: String,
        masks: Vec<Node>
    }
);

impl FileLayer {
    getter_func!(pub scale -> bool);
    getter_func!(pub scaling_method -> u32);
    getter_func!(pub scaling_filter -> &str);
    getter_func!(pub source -> &Path);
}

make_node!(
    #[doc = "Filter layer."]
    FilterLayer,
    FilterLayerProps,
    {
        filter_name: String,
        filter_version: u32,
        channel_flags: String,
        collapsed: bool,
        composite_op: CompositeOp,
        opacity: u8,
        masks: Vec<Node>
    }
);

make_node!(
    #[doc = "Fill layer, also known as `generatorlayer`."]
    FillLayer,
    FillLayerProps,
    {
        opacity: u8,
        composite_op: CompositeOp,
        generator_name: String,
        generator_version: u32,
        channel_flags: String,
        collapsed: bool,
        masks: Vec<Node>
    }
);

impl FillLayer {
    getter_func!(pub generator_version -> u32);
    getter_func!(pub generator_name -> &str);
}

make_node!(
    #[doc = "Clone layer."]
    CloneLayer,
    CloneLayerProps,
    {
        clone_type: u32,
        clone_from: String,
        composite_op: CompositeOp,
        opacity: u8,
        clone_from_uuid: Uuid,
        channel_flags: String,
        collapsed: bool,
        masks: Vec<Node>
    }
);

impl CloneLayer {
    getter_func!(pub clone_type -> u32);
    getter_func!(pub clone_from_uuid -> &Uuid);
    getter_func!(pub clone_from -> &str);
}

make_node!(
    #[doc = "Transparency mask."]
    TransparencyMask,
    TransparencyMaskProps,
    {}
);

make_node!(
    #[doc = "Transform mask."]
    TransformMask,
    TransformMaskProps,
    {}
);

make_node!(
    #[doc = "Colorize mask."]
    ColorizeMask,
    ColorizeMaskProps,
    {
        limit_to_device: bool,
        show_coloring: bool,
        cleanup: u8,
        use_edge_detection: bool,
        edge_detection_size: u32,
        fuzzy_radius: u32,
        edit_keystrokes: bool,
        composite_op: CompositeOp,
        colorspace: Colorspace
    }
);

impl ColorizeMask {
    getter_func!(pub limit_to_device -> bool);
    getter_func!(pub show_coloring -> bool);
    getter_func!(pub cleanup -> u8);
    getter_func!(pub use_edge_detection -> bool);
    getter_func!(pub edge_detection_size -> u32);
    getter_func!(pub fuzzy_radius -> u32);
    getter_func!(pub edit_keystrokes -> bool);
}

make_node!(
    #[doc = "Vector layer, also known as `shapelayer`."]
    VectorLayer,
    VectorLayerProps,
    {
        composite_op: CompositeOp,
        opacity: u8,
        channel_flags: String,
        collapsed: bool,
        masks: Vec<Node>
    }
);

make_node!(
    #[doc = "Group layer."]
    GroupLayer,
    GroupLayerProps,
    {
        composite_op: CompositeOp,
        collapsed: bool,
        passthrough: bool,
        opacity: u8,
        layers: Vec<Node>
    }
);

impl GroupLayer {
    getter_func!(pub layers -> &[Node]);
    getter_func!(pub passthrough -> bool);
}

#[derive(ParseTag)]
pub(crate) struct FilterMaskProps {
    #[XmlAttr(
        qname = "filtername",
        pre_parse = "unescape_value()?",
        fun_override = "filter_name.to_string()"
    )]
    filter_name: String,
    #[XmlAttr(qname = "filterversion")]
    filter_version: u32,
}

#[derive(ParseTag)]
pub(crate) struct PaintLayerProps {
    #[XmlAttr(qname = "compositeop")]
    composite_op: CompositeOp,
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
    #[XmlAttr(extract_data = false, fun_override = "Vec::<Node>::new()")]
    masks: Vec<Node>,
}

#[derive(ParseTag)]
pub(crate) struct SelectionMaskProps {
    #[XmlAttr(fun_override = "parse_bool(active)?")]
    active: bool,
}

#[derive(ParseTag)]
pub(crate) struct FileLayerProps {
    #[XmlAttr(fun_override = "parse_bool(collapsed)?")]
    collapsed: bool,
    //TODO: enum
    #[XmlAttr(
        qname = "scalingfilter",
        pre_parse = "unescape_value()?.into()",
        fun_override = "scaling_filter"
    )]
    scaling_filter: String,
    // this bool is has value "true" or "false", instead of 1 or 0
    scale: bool,
    #[XmlAttr(qname = "compositeop")]
    composite_op: CompositeOp,
    opacity: u8,
    #[XmlAttr(
        qname = "colorspacename",
        pre_parse = "unescape_value()?",
        fun_override = "Colorspace::try_from(colorspace.as_ref())?"
    )]
    colorspace: Colorspace,
    //TODO: figure out correct type
    #[XmlAttr(qname = "scalingmethod")]
    scaling_method: u32,
    #[XmlAttr(
        qname = "source",
        pre_parse = "unescape_value()?.to_string().into()",
        fun_override = "source"
    )]
    source: PathBuf,
    #[XmlAttr(
        qname = "channelflags",
        pre_parse = "unescape_value()?.into()",
        fun_override = "channel_flags"
    )]
    channel_flags: String,
    #[XmlAttr(extract_data = false, fun_override = "Vec::<Node>::new()")]
    masks: Vec<Node>,
}

#[derive(ParseTag)]
pub(crate) struct FilterLayerProps {
    #[XmlAttr(
        qname = "filtername",
        pre_parse = "unescape_value()?",
        fun_override = "filter_name.to_string()"
    )]
    filter_name: String,
    #[XmlAttr(qname = "filterversion")]
    filter_version: u32,
    #[XmlAttr(
        qname = "channelflags",
        pre_parse = "unescape_value()?.into()",
        fun_override = "channel_flags"
    )]
    channel_flags: String,
    #[XmlAttr(fun_override = "parse_bool(collapsed)?")]
    collapsed: bool,
    #[XmlAttr(qname = "compositeop")]
    composite_op: CompositeOp,
    opacity: u8,
    #[XmlAttr(extract_data = false, fun_override = "Vec::<Node>::new()")]
    masks: Vec<Node>,
}

#[derive(ParseTag)]
pub(crate) struct FillLayerProps {
    opacity: u8,
    #[XmlAttr(qname = "compositeop")]
    composite_op: CompositeOp,
    //TODO: enum?
    #[XmlAttr(
        qname = "generatorname",
        pre_parse = "unescape_value()?.into()",
        fun_override = "generator_name"
    )]
    generator_name: String,
    #[XmlAttr(qname = "generatorversion")]
    generator_version: u32,
    #[XmlAttr(
        qname = "channelflags",
        pre_parse = "unescape_value()?.into()",
        fun_override = "channel_flags"
    )]
    channel_flags: String,
    #[XmlAttr(fun_override = "parse_bool(collapsed)?")]
    collapsed: bool,
    #[XmlAttr(extract_data = false, fun_override = "Vec::<Node>::new()")]
    masks: Vec<Node>,
}

#[derive(ParseTag)]
pub(crate) struct CloneLayerProps {
    //TODO: figure out proper type
    #[XmlAttr(qname = "clonetype")]
    clone_type: u32,
    #[XmlAttr(
        qname = "clonefrom",
        pre_parse = "unescape_value()?.into()",
        fun_override = "clone_from"
    )]
    clone_from: String,
    #[XmlAttr(qname = "compositeop")]
    composite_op: CompositeOp,
    opacity: u8,
    #[XmlAttr(
        qname = "clonefromuuid",
        pre_parse = "unescape_value()?",
        fun_override = "Uuid::from_str(clone_from_uuid.as_ref())?"
    )]
    clone_from_uuid: Uuid,
    #[XmlAttr(
        qname = "channelflags",
        pre_parse = "unescape_value()?.into()",
        fun_override = "channel_flags"
    )]
    channel_flags: String,
    #[XmlAttr(fun_override = "parse_bool(collapsed)?")]
    collapsed: bool,
    #[XmlAttr(extract_data = false, fun_override = "Vec::<Node>::new()")]
    masks: Vec<Node>,
}

#[derive(ParseTag)]
pub(crate) struct ColorizeMaskProps {
    #[XmlAttr(
        qname = "limit-to-device",
        fun_override = "parse_bool(limit_to_device)?"
    )]
    limit_to_device: bool,
    #[XmlAttr(qname = "show-coloring", fun_override = "parse_bool(show_coloring)?")]
    show_coloring: bool,
    //TODO: is it a proper type?
    cleanup: u8,
    #[XmlAttr(
        qname = "use-edge-detection",
        fun_override = "parse_bool(use_edge_detection)?"
    )]
    use_edge_detection: bool,
    #[XmlAttr(qname = "edge-detection-size")]
    edge_detection_size: u32,
    #[XmlAttr(qname = "fuzzy-radius")]
    fuzzy_radius: u32,
    #[XmlAttr(
        qname = "edit-keystrokes",
        fun_override = "parse_bool(edit_keystrokes)?"
    )]
    edit_keystrokes: bool,
    #[XmlAttr(qname = "compositeop")]
    composite_op: CompositeOp,
    #[XmlAttr(
        qname = "colorspacename",
        pre_parse = "unescape_value()?",
        fun_override = "Colorspace::try_from(colorspace.as_ref())?"
    )]
    colorspace: Colorspace,
}

#[derive(ParseTag)]
pub(crate) struct VectorLayerProps {
    #[XmlAttr(qname = "compositeop")]
    composite_op: CompositeOp,
    opacity: u8,
    #[XmlAttr(
        qname = "channelflags",
        pre_parse = "unescape_value()?.into()",
        fun_override = "channel_flags"
    )]
    channel_flags: String,
    #[XmlAttr(fun_override = "parse_bool(collapsed)?")]
    collapsed: bool,
    #[XmlAttr(extract_data = false, fun_override = "Vec::<Node>::new()")]
    masks: Vec<Node>,
}

// No props beyond common ones
pub(crate) struct TransparencyMaskProps();

impl TransparencyMaskProps {
    pub(crate) fn new() -> TransparencyMaskProps {
        TransparencyMaskProps()
    }
}

// Same here
pub(crate) struct TransformMaskProps();

impl TransformMaskProps {
    pub(crate) fn new() -> TransformMaskProps {
        TransformMaskProps()
    }
}

#[derive(Debug, ParseTag)]
#[ExtraArgs(
    extra_args = "reader: &mut quick_xml::Reader<&[u8]>, conf: crate::parse::ParsingConfiguration"
)]
pub(crate) struct GroupLayerProps {
    #[XmlAttr(qname = "compositeop")]
    pub(crate) composite_op: CompositeOp,
    #[XmlAttr(fun_override = "parse_bool(collapsed)?")]
    pub(crate) collapsed: bool,
    #[XmlAttr(fun_override = "parse_bool(passthrough)?")]
    pub(crate) passthrough: bool,
    pub(crate) opacity: u8,
    #[XmlAttr(
        extract_data = false,
        fun_override = "crate::parse::get_layers(reader, conf, true)?"
    )]
    pub(crate) layers: Vec<Node>,
}

#[derive(ParseTag)]
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
    colorlabel: u32,
    // TODO: find why these were u32 originally
    y: i32,
    x: i32,
    #[XmlAttr(
        qname = "intimeline",
        pre_parse = "unescape_value()?",
        fun_override = "parse_in_timeline(in_timeline.as_ref(), tag)?"
    )]
    in_timeline: InTimeline,
}

// TODO: move these out

/// Visibility of a node in the timeline.
#[derive(Debug, Clone, Copy)]
pub enum InTimeline {
    /// Node is visible in timeline.
    True(Onionskin),
    /// Node is not visible.
    False,
}

/// Whether onionskinning is enabled.
pub type Onionskin = bool;

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
            )));
        }
    }
}
