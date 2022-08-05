use syn::parse_quote;

use crate::model::Model;

use super::struct_def::Struct;

pub struct NewMethod {
    fields: Vec<syn::FieldValue>,
    vis: syn::Visibility,

}

fn ds_value(ident: syn::Ident) -> syn::FieldValue {
    syn::FieldValue {
        attrs: Vec::new(),
        member: syn::Member::Named(ident),
        colon_token: None, // shothand so no colon
        expr: parse_quote!("ds"),
    }
}

fn as_len_value(ident: syn::Ident) -> syn::FieldValue {
    let colon: syn::token::Colon = syn::Token![:](proc_macro2::Span::call_site());
    syn::FieldValue {
        attrs: Vec::new(),
        member: syn::Member::Named(ident),
        colon_token: Some(colon),
        expr: parse_quote!(std::sync::Arc::new(std::sync::atomics::AtomicUsize::new(0))),
    }
}

impl NewMethod {
    pub fn from(model: &Model, struct_def: &Struct) -> Self {
        let fields: Vec<_> = struct_def
            .vars
            .iter()
            .map(|def| def.ident.clone())
            .map(Option::unwrap)
            .map(as_len_value)
            .collect();
        Self {
            fields,
            vis: model.vis.clone(),
        }

    }
}
