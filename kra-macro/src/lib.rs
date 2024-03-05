use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Field};

//TODO: make it look pretty and test properly

#[proc_macro_derive(ParseTag, attributes(XmlAttr))]
pub fn parse_tag(item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as DeriveInput);
    let ident = item.ident;
    let fields = match item {
        DeriveInput {
            data: Data::Struct(item),
            ..
        } => item.fields,
        _ => panic!("expected a struct"),
    };
    let fields = match fields {
        syn::Fields::Named(fields) => fields,
        _ => panic!("expected a struct with named fields"),
    }
    .named;
    // Two interleaved parts - one is all get_attr(), other are fields in struct construction
    let tokens: Vec<TokenStream2> = fields
        .iter()
        .map(|item| gen_get_attr(item))
        .flatten()
        .collect();
    let tokens_first = tokens.iter().step_by(2);
    let tokens_second = tokens.iter().skip(1).step_by(2);
    quote! {
        impl #ident {
            pub(crate) fn parse_tag(tag: &BytesStart) -> Result<Self, MetadataErrorReason> {
                #( #tokens_first )*
                Ok(#ident {
                    #( #tokens_second ),*
                })
            }
        }
    }
    .into()
}

fn gen_get_attr(item: &Field) -> [TokenStream2; 2] {
    let ident = item.ident.as_ref().unwrap();
    //Attribute of the field
    let attr = item
        .attrs
        .iter()
        .map(|x| XmlAttr::from_meta(&x.meta))
        .find(|x| x.is_ok())
        // If the attribute is present and correct, all is well.
        // If it is not present or is incorrect, it will not be parsed.
        // There is no way to discern between incorrect attribute or any other
        // non-applicable one.
        // Therefore unwrap()
        // TODO: find a better way?
        // TODO: better error message (the current one is incorrect for wrong fun_override)
        // For this, I could search for existence of XmlAttr attribute, regardless of whether
        // it was successfully parsed. That would indicate that the user made a mistake instead
        // of not adding the attribute at all.
        .expect(format!("expected XmlAttr attribute on field {}", ident).as_str())
        .unwrap();
    let qname = attr.qname.unwrap_or(ident.to_string());
    //TODO: remove requirement for function override when default parsing is implemented
    let fun_override = attr
        .fun_override
        .expect("currently, defining fun_override is necessary");
    let pre_parse = attr.pre_parse;
    let fun_override: syn::Expr =
        syn::parse_str(fun_override.as_str()).expect("could not parse function override");
    //TODO: default parsing behaviour
    // First part of output - statement to get attribute from XML
    // let conversion_function = match item.ty {
    //     //TODO: find better way to check the type
    //     syn::Type::Path(ref path) if path.to_token_stream() == "bool".into() => {
    //     }
    //     //TODO: better error message (check if function override was provided?)
    //     _ => panic!("type not supported for parsing"),
    // };
    let tokens_first = match pre_parse {
        Some(pre_parse) => {
            let pre_parse: syn::Expr =
                syn::parse_str(pre_parse.as_str()).expect("could not parse pre-parsing code");
            quote! {
                let #ident = event_get_attr(&tag, #qname)?.#pre_parse;
            }
        }
        None => quote! {
            let #ident = event_get_attr(&tag, #qname)?;
        },
    };
    // let tokens_first = quote! {
    //     let #ident = event_get_attr(&tag, #qname)?;
    // };
    // TODO: replace fun_override with a parser that is chosen beforehand (default or override)
    let tokens_second = quote! {
        #ident: #fun_override
    };
    [tokens_first, tokens_second]
}

// Attribute which stores qname of a struct field
#[derive(Debug, FromMeta)]
pub(crate) struct XmlAttr {
    // QName of the attribute
    // Default is to reuse field name
    pub(crate) qname: Option<String>,
    // Parsing function override
    // The string is parsed and then inserted as-is
    #[darling(default)]
    pub(crate) fun_override: Option<String>,
    pub(crate) pre_parse: Option<String>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn pass() {
        let t = trybuild::TestCases::new();
        t.pass("tests/00-typical-use.rs");
    }
}
