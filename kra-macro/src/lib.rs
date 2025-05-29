use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Field};

#[proc_macro_derive(ParseTag, attributes(XmlAttr, ExtraArgs))]
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
    // Extra args
    let extra_args: TokenStream2 = syn::parse_str(&match item
        .attrs
        .iter()
        .map(|x| ExtraArgs::from_meta(&x.meta))
        .find(|x| x.is_ok())
    {
        Some(Ok(ExtraArgs {
            extra_args: Some(args),
        })) => format!(", {}", args),
        _ => String::default(),
    })
    .unwrap_or(TokenStream2::default());
    quote! {
        impl #ident {
            pub(crate) fn parse_tag(tag: &BytesStart #extra_args) -> Result<Self, MetadataErrorReason> {
                #( #tokens_first )*
                Ok(#ident {
                    #( #tokens_second ),*
                })
            }
        }
    }
    .into()
}

// TODO: try to not convert items to strings in order to deal with hygiene issues
fn gen_get_attr(item: &Field) -> [TokenStream2; 2] {
    let ident = item.ident.as_ref().unwrap();
    //Attribute of the field
    let attr = item
        .attrs
        .iter()
        .map(|x| XmlAttr::from_meta(&x.meta))
        .find(|x| x.is_ok())
        // TODO: better error message (the current one is incorrect for wrong fun_override)
        // For this, I could search for existence of XmlAttr attribute, regardless of whether
        // it was successfully parsed. That would indicate that the user made a mistake instead
        // of not adding the attribute at all.
        .unwrap_or(Ok(XmlAttr::default()))
        .unwrap();
    let qname = attr.qname.unwrap_or(ident.to_string());
    let fun_override = attr
        .fun_override
        .unwrap_or(format!("parse_attr({})?", ident));
    let pre_parse = attr.pre_parse;
    let extract_data = attr.extract_data;
    let fun_override: syn::Expr =
        syn::parse_str(fun_override.as_str()).expect("could not parse function override");

    // First part of output - statement to get attribute from XML
    let tokens_first = match (extract_data, pre_parse) {
        (Some(false), _) => quote! {
            let #ident = #fun_override;
        },
        (_, Some(pre_parse)) => {
            let pre_parse: syn::Expr =
                syn::parse_str(pre_parse.as_str()).expect("could not parse pre-parsing code");
            quote! {
                let #ident = event_get_attr(&tag, #qname)?.#pre_parse;
            }
        }
        (_, None) => quote! {
            let #ident = event_get_attr(&tag, #qname)?;
        },
    };
    let tokens_second = quote! {
        #ident: #fun_override
    };
    [tokens_first, tokens_second]
}

// Attribute which stores qname of a struct field
#[derive(Debug, Default, FromMeta)]
pub(crate) struct XmlAttr {
    // QName of the attribute
    // Default is to reuse field name
    pub(crate) qname: Option<String>,
    // Parsing function override
    // The string is parsed and then inserted as-is
    #[darling(default)]
    pub(crate) fun_override: Option<String>,
    // Code to append immediately after data extraction (like unescape_value())
    pub(crate) pre_parse: Option<String>,
    // Do not extract data, run function in fun_override instead
    pub(crate) extract_data: Option<bool>,
}

// Attribute to add extra arguments for the resulting function
#[derive(Debug, FromMeta)]
pub(crate) struct ExtraArgs {
    pub(crate) extra_args: Option<String>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn pass() {
        let t = trybuild::TestCases::new();
        t.pass("tests/00-typical-use.rs");
    }
}
