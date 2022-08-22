use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::{parse_quote, Pat, PathArguments, Token};

use crate::model::{Field, Model, Wrapper};

use super::struct_def::Struct;

pub struct NewMethod {
    pub locals: Vec<syn::Local>,
    pub fields: Vec<syn::FieldValue>,
    pub vis: syn::Visibility,
}

fn as_len_value(ident: syn::Ident) -> syn::FieldValue {
    let colon: syn::token::Colon = syn::Token![:](Span::call_site());
    syn::FieldValue {
        attrs: Vec::new(),
        member: syn::Member::Named(ident),
        colon_token: Some(colon),
        expr: parse_quote!(std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0))),
    }
}

fn len_expr(ty: &syn::Type, prefix: u8) -> Box<syn::Expr> {
    let next_prefix = prefix + 1;
    let expr: syn::Expr = parse_quote!(
    dbstruct::traits::data_store::Orderd::get_lt(&ds, &#next_prefix)?
        .map(|(len, _): (u8, #ty)| len)
        .unwrap_or(0)
    );
    Box::new(expr)
}

fn len_init(field: &Field) -> Option<syn::Local> {
    let ty = match &field.wrapper {
        Wrapper::Vec { ty } => ty,
        _ => return None,
    };

    let ident = syn::PathSegment {
        ident: field.ident.clone(),
        arguments: PathArguments::None,
    };
    let ident = syn::Path {
        leading_colon: None,
        segments: Punctuated::from_iter(std::iter::once(ident)),
    };
    let ident = syn::PatPath {
        attrs: Vec::new(),
        qself: None,
        path: ident,
    };
    let expr = len_expr(ty, field.key);
    let eq_token = Token![=](Span::call_site());
    Some(syn::Local {
        attrs: Vec::new(),
        let_token: Token![let](Span::call_site()),
        pat: Pat::Path(ident),
        init: Some((eq_token, expr)),
        semi_token: Token![;](Span::call_site()),
    })
}

impl NewMethod {
    pub fn from(model: &Model, struct_def: &Struct) -> Self {
        let locals: Vec<_> = model
            .fields
            .iter()
            .filter_map(|field| len_init(field))
            .collect();
        let fields: Vec<_> = struct_def
            .len_vars
            .iter()
            .map(|def| def.ident.clone())
            .map(Option::unwrap)
            .map(as_len_value)
            .collect();
        Self {
            locals,
            fields,
            vis: model.vis.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_method_has_one_local() {
        let model = Model::mock_vec();
        let struct_def = Struct::from(&model);
        let new_method = NewMethod::from(&model, &struct_def);
        assert!(new_method.locals.len() == 1);
    }
}
