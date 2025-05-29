#![allow(unreachable_expression)]

use kra_macro::ParseTag;

#[derive(ParseTag)]
struct Thing {
    #[XmlAttr(qname = "author", fun_override = "todo!()")]
    author: String,
    #[XmlAttr(qname = "x", fun_override = "todo!()")]
    x: u32,
    #[XmlAttr(qname = "y", fun_override = "todo!()")]
    y: u32,
    #[XmlAttr(qname = "on", fun_override = "override_bool(on)")]
    on: bool,
}

struct BytesStart();

struct MetadataErrorReason();

struct Attribute();

fn event_get_attr(_tag: &BytesStart, _name: &str) -> Result<Attribute, MetadataErrorReason> {
    todo!()
}

fn override_bool(item: Attribute) -> bool {
    todo!()
}

fn main() {}
