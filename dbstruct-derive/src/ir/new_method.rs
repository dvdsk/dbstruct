use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::{parse_quote, Pat, PathArguments, Token};

use crate::model::backend::Backend;
use crate::model::{Field, Model, Wrapper};

use super::struct_def::{as_len_ident, Struct};

pub struct NewMethod {
    pub locals: Vec<syn::Local>,
    pub fields: Vec<syn::FieldValue>,
    pub vis: syn::Visibility,
    pub arg: Option<syn::FnArg>,
    pub error_ty: syn::Type,
}

fn as_len_value(ident: syn::Ident) -> syn::FieldValue {
    let colon: syn::token::Colon = syn::Token![:](Span::call_site());
    syn::FieldValue {
        attrs: Vec::new(),
        member: syn::Member::Named(ident.clone()),
        colon_token: Some(colon),
        expr: parse_quote!(std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(#ident))),
    }
}

fn len_expr(ty: &syn::Type, prefix: u8) -> Box<syn::Expr> {
    let expr: syn::Expr = parse_quote!(
        ::dbstruct::traits::data_store::Orderd::get_lt(
                &ds,
                &::dbstruct::wrappers::Prefixed::max(#prefix),
            )?
            .map(|(key, _): (::dbstruct::wrappers::Prefixed, #ty)| key)
            .map(|key| key.index() + 1) // a vecs len is index + 1
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
        ident: as_len_ident(&field.ident),
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

fn sled_from_path() -> syn::Local {
    let stmt: syn::Stmt = parse_quote!(
    let ds = ::dbstruct::sled::Config::default()
        .path(path)
        .open()?
        .open_tree("DbStruct")?;
    );
    match stmt {
        syn::Stmt::Local(local) => local,
        _ => unreachable!(),
    }
}

fn hashmap() -> syn::Local {
    let stmt: syn::Stmt = parse_quote!(
    let ds = ::dbstruct::stores::HashMap::new();
    );
    match stmt {
        syn::Stmt::Local(local) => local,
        _ => unreachable!(),
    }
}

impl NewMethod {
    pub fn from(model: &Model, struct_def: &Struct) -> Self {
        let fields: Vec<_> = struct_def
            .len_vars
            .iter()
            .map(|def| def.ident.clone())
            .map(Option::unwrap)
            .map(as_len_value)
            .collect();

        let mut locals = Vec::new();

        let arg;
        let error_ty;
        match model.backend {
            Backend::Sled => {
                locals.push(sled_from_path());
                arg = Some(parse_quote!(path: &std::path::Path));
                error_ty = parse_quote!(::dbstruct::sled::Error);
            }
            Backend::HashMap => {
                locals.push(hashmap());
                arg = None;
                error_ty = parse_quote!(::dbstruct::stores::HashMapError);
            }
            Backend::Trait { .. } => {
                arg = Some(parse_quote!(ds: DS));
                error_ty = parse_quote!(DS::Error);
            }
            #[cfg(test)]
            Backend::Test => unreachable!("test not used in new method"),
        };

        let inits = model.fields.iter().filter_map(|field| len_init(field));
        locals.extend(inits);

        Self {
            locals,
            fields,
            vis: model.vis.clone(),
            arg,
            error_ty,
        }
    }
}

#[cfg(test)]
mod tests {
    use quote::ToTokens;

    use super::*;

    #[test]
    fn has_one_local() {
        let model = Model::mock_vec();
        let struct_def = Struct::from(&model);
        let new_method = NewMethod::from(&model, &struct_def);
        assert!(new_method.locals.len() == 2);
    }

    #[test]
    fn body_is_valid_rust() {
        let model = Model::mock_vec();
        let struct_def = Struct::from(&model);
        let new_method = NewMethod::from(&model, &struct_def);

        let stmts: Vec<_> = new_method
            .locals
            .into_iter()
            .map(syn::Stmt::Local)
            .collect();
        let block = syn::Block {
            brace_token: syn::token::Brace(proc_macro2::Span::call_site()),
            stmts,
        };
        let tokens = block.to_token_stream();
        println!("{tokens}");
        assert!(syn::parse2::<syn::Block>(tokens).is_ok())
    }
}
