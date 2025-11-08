use itertools::Itertools;
use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::{parse_quote, Expr, Ident, LocalInit, Pat, PathArguments, Token};

use crate::model::backend::Backend;
use crate::model::{Field, Model, Wrapper};

use super::struct_def::{
    deque_head_ident, deque_tail_ident, no_syn_phantom_ident, vec_len_ident, Struct,
};

pub struct NewMethod {
    pub locals: Vec<syn::Local>,
    pub members: Vec<syn::FieldValue>,
    pub vis: syn::Visibility,
    pub arg: Option<syn::FnArg>,
    pub error_ty: syn::Type,
    pub name: syn::Ident,
}

fn as_member(field: &syn::Field) -> syn::FieldValue {
    let colon: syn::token::Colon = syn::Token![:](Span::call_site());
    let ident = field.ident.as_ref().expect("is Some");
    syn::FieldValue {
        attrs: Vec::new(),
        member: syn::Member::Named(ident.clone()),
        colon_token: Some(colon),
        expr: parse_quote!(#ident),
    }
}

fn len_expr(ty: &syn::Type, prefix: u8) -> Box<syn::Expr> {
    let expr: syn::Expr = parse_quote!(
        std::sync::Arc::new(
            std::sync::atomic::AtomicUsize::new(
                ::dbstruct::traits::data_store::Ordered::get_lt(
                        &ds,
                        &::dbstruct::wrapper::VecPrefixed::max(#prefix),
                    )?
                    .map(|(key, _): (::dbstruct::wrapper::VecPrefixed, #ty)| key)
                    .map(|key| key.index() + 1) // a vecs len is index + 1
                    .unwrap_or(0)
            ) // atomic new
        ) // arc new
    );
    Box::new(expr)
}

fn tail_expr(ty: &syn::Type, prefix: u8) -> Box<syn::Expr> {
    let expr: syn::Expr = parse_quote!(
        std::sync::Arc::new(
            std::sync::atomic::AtomicU64::new(
                ::dbstruct::traits::data_store::Ordered::get_lt(
                        &ds,
                        &::dbstruct::wrapper::DequePrefixed::max(#prefix),
                    )?
                    .map(|(key, _): (::dbstruct::wrapper::DequePrefixed, #ty)| key)
                    .map(|key| key.index() + 1)
                    .unwrap_or(u64::MAX / 2)
            ) // atomic new
        ) // arc new
    );
    Box::new(expr)
}

fn head_expr(ty: &syn::Type, prefix: u8) -> Box<syn::Expr> {
    let expr: syn::Expr = parse_quote!(
        std::sync::Arc::new(
            std::sync::atomic::AtomicU64::new(
                ::dbstruct::traits::data_store::Ordered::get_gt(
                        &ds,
                        &::dbstruct::wrapper::DequePrefixed::min(#prefix),
                    )?
                    .map(|(key, _): (::dbstruct::wrapper::DequePrefixed, #ty)| key)
                    .map(|key| key.index() - 1)
                    .unwrap_or(u64::MAX / 2 - 1)
            ) // atomic new
        ) // arc new
    );
    Box::new(expr)
}

fn local_init(expr: Box<Expr>, ident: Ident) -> syn::Local {
    let ident = syn::PathSegment {
        ident,
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
    let eq_token = Token![=](Span::call_site());
    syn::Local {
        attrs: Vec::new(),
        let_token: Token![let](Span::call_site()),
        pat: Pat::Path(ident),
        init: Some(LocalInit {
            eq_token,
            expr,
            diverge: None,
        }),
        semi_token: Token![;](Span::call_site()),
    }
}

fn vec_len_init(field: &Field) -> syn::Local {
    let ty = match &field.wrapper {
        Wrapper::Vec { ty } => ty,
        _ => unreachable!("checked by caller"),
    };

    local_init(len_expr(ty, field.key), vec_len_ident(&field.ident))
}

fn deque_head_init(field: &Field) -> syn::Local {
    let ty = match &field.wrapper {
        Wrapper::VecDeque { ty } => ty,
        _ => unreachable!("checked by caller"),
    };

    local_init(head_expr(ty, field.key), deque_head_ident(&field.ident))
}

fn deque_tail_init(field: &Field) -> syn::Local {
    let ty = match &field.wrapper {
        Wrapper::VecDeque { ty } => ty,
        _ => unreachable!("checked by caller"),
    };

    local_init(tail_expr(ty, field.key), deque_tail_ident(&field.ident))
}

fn no_sync_phantom_local_init() -> syn::Local {
    let ident = syn::PathSegment {
        ident: no_syn_phantom_ident(),
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
    let eq_token = Token![=](Span::call_site());
    syn::Local {
        attrs: Vec::new(),
        let_token: Token![let](Span::call_site()),
        pat: Pat::Path(ident),
        init: Some(LocalInit {
            eq_token,
            expr: parse_quote!(::std::marker::PhantomData),
            diverge: None,
        }),
        semi_token: Token![;](Span::call_site()),
    }
}

struct DbSetup {
    local: Option<syn::Local>,
    arg: Option<syn::FnArg>,
    error_ty: syn::Type,
    name: &'static str,
}

fn sled_from_path() -> DbSetup {
    let stmt: syn::Stmt = parse_quote!(
    let ds = ::dbstruct::sled::Config::default()
        .path(path)
        .open().map_err(::dbstruct::Error::Database)?
        .open_tree("DbStruct").map_err(::dbstruct::Error::Database)?;
    );
    let local = match stmt {
        syn::Stmt::Local(local) => local,
        _ => unreachable!(),
    };

    DbSetup {
        local: Some(local),
        arg: Some(parse_quote!(path: impl AsRef<std::path::Path>)),
        error_ty: parse_quote!(::dbstruct::sled::Error),
        name: "open_path",
    }
}

fn sled_from_db() -> DbSetup {
    let stmt: syn::Stmt = parse_quote!(
        let ds = db.open_tree("DbStruct").map_err(::dbstruct::Error::Database)?;
    );
    let local = match stmt {
        syn::Stmt::Local(local) => local,
        _ => unreachable!(),
    };

    DbSetup {
        local: Some(local),
        arg: Some(parse_quote!(db: ::dbstruct::sled::Db)),
        error_ty: parse_quote!(::dbstruct::sled::Error),
        name: "open_db",
    }
}

fn sled_from_tree() -> DbSetup {
    let stmt: syn::Stmt = parse_quote!(
        let ds = tree;
    );
    let local = match stmt {
        syn::Stmt::Local(local) => local,
        _ => unreachable!(),
    };

    DbSetup {
        local: Some(local),
        arg: Some(parse_quote!(tree: ::dbstruct::sled::Tree)),
        error_ty: parse_quote!(::dbstruct::sled::Error),
        name: "open_tree",
    }
}

fn hashmap() -> DbSetup {
    let stmt: syn::Stmt = parse_quote!(
        let ds = ::dbstruct::stores::HashMap::new();
    );
    let local = match stmt {
        syn::Stmt::Local(local) => local,
        _ => unreachable!(),
    };

    DbSetup {
        local: Some(local),
        arg: None,
        error_ty: parse_quote!(::dbstruct::stores::HashMapError),
        name: "new",
    }
}

fn btreemap() -> DbSetup {
    let stmt: syn::Stmt = parse_quote!(
    let ds = ::dbstruct::stores::BTreeMap::new();
    );
    let local = match stmt {
        syn::Stmt::Local(local) => local,
        _ => unreachable!(),
    };

    DbSetup {
        local: Some(local),
        arg: None,
        error_ty: parse_quote!(::dbstruct::stores::BTreeMapError),
        name: "new",
    }
}

fn for_trait() -> DbSetup {
    DbSetup {
        local: None,
        arg: Some(parse_quote!(ds: DS)),
        error_ty: parse_quote!(DS::DbError),
        name: "open",
    }
}

impl NewMethod {
    pub fn list(model: &Model, struct_def: &Struct) -> Vec<Self> {
        match model.backend {
            Backend::Sled => vec![sled_from_db(), sled_from_path(), sled_from_tree()],
            Backend::HashMap => {
                vec![hashmap()]
            }
            Backend::BTreeMap => {
                vec![btreemap()]
            }
            Backend::Trait { .. } => {
                vec![for_trait()]
            }
            #[cfg(test)]
            Backend::Test => unreachable!("test not used in new method"),
        }
        .into_iter()
        .map(|db| db_setup_to_new_method(db, model, struct_def))
        .collect_vec()
    }
}

fn db_setup_to_new_method(db: DbSetup, model: &Model, struct_def: &Struct) -> NewMethod {
    let DbSetup {
        local,
        arg,
        error_ty,
        name,
    } = db;

    let mut locals = local.into_iter().collect_vec();
    let inits = model.fields.iter().flat_map(|field| match &field.wrapper {
        Wrapper::Vec { .. } => [vec_len_init(field)].to_vec(),
        Wrapper::VecDeque { .. } => [deque_head_init(field), deque_tail_init(field)].to_vec(),
        _ => Vec::new(),
    });
    locals.extend(inits);
    locals.push(no_sync_phantom_local_init());

    NewMethod {
        locals,
        members: struct_def
            .member_vars
            .iter()
            .map(as_member)
            .collect(),
        vis: model.vis.clone(),
        arg,
        error_ty,
        name: syn::Ident::new(name, struct_def.ident.span()),
    }
}

#[cfg(test)]
mod tests {
    use quote::ToTokens;

    use super::*;

    #[test]
    fn adds_one_local() {
        let model = Model::mock_vec();
        let struct_def = Struct::from(&model);
        let new_methods = NewMethod::list(&model, &struct_def);
        assert_eq!(new_methods[0].locals.len(), 3);
    }

    #[test]
    fn adds_two_local() {
        let model = Model::mock_vecdeque();
        let struct_def = Struct::from(&model);
        let new_methods = NewMethod::list(&model, &struct_def);
        assert_eq!(new_methods[0].locals.len(), 4);
    }

    #[test]
    fn bodies_are_valid_rust() {
        let model = Model::mock_vec();
        let struct_def = Struct::from(&model);
        for new_method in NewMethod::list(&model, &struct_def) {
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

    #[test]
    fn three_factories_for_sled() {
        let model = Model::mock_vecdeque(); // uses sled
        let struct_def = Struct::from(&model);
        let new_methods = NewMethod::list(&model, &struct_def);
        assert_eq!(new_methods.len(), 3);
    }
}
