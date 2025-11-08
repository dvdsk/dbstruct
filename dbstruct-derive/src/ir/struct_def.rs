use syn::parse_quote;

use crate::model::backend::Backend;
use crate::model::{Field, Model};

pub struct Struct {
    pub ident: syn::Ident,
    pub vis: syn::Visibility,
    /// Extra variables such as the current length
    /// of the vector wrapper
    pub member_vars: Vec<syn::Field>,
    pub db: syn::Field,
}

pub fn vec_len_ident(ident: &syn::Ident) -> syn::Ident {
    let name = format!("{}_len", ident);
    syn::Ident::new(&name, proc_macro2::Span::call_site())
}

fn vec_len_field(field: &Field) -> syn::Field {
    syn::Field {
        attrs: Vec::new(),
        vis: syn::Visibility::Inherited,
        ident: Some(vec_len_ident(&field.ident)),
        colon_token: None,
        ty: parse_quote!(std::sync::Arc<std::sync::atomic::AtomicUsize>),
        mutability: syn::FieldMutability::None,
    }
}

pub fn deque_head_ident(ident: &syn::Ident) -> syn::Ident {
    let name = format!("{}_head", ident);
    syn::Ident::new(&name, proc_macro2::Span::call_site())
}

pub fn deque_tail_ident(ident: &syn::Ident) -> syn::Ident {
    let name = format!("{}_tail", ident);
    syn::Ident::new(&name, proc_macro2::Span::call_site())
}

pub fn no_syn_phantom_ident() -> syn::Ident {
    syn::Ident::new("prevent_sync", proc_macro2::Span::call_site())
}

fn deque_head_field(field: &Field) -> syn::Field {
    syn::Field {
        attrs: Vec::new(),
        vis: syn::Visibility::Inherited,
        ident: Some(deque_head_ident(&field.ident)),
        colon_token: None,
        ty: parse_quote!(::std::sync::Arc<::std::sync::atomic::AtomicU64>),
        mutability: syn::FieldMutability::None,
    }
}

fn deque_tail_field(field: &Field) -> syn::Field {
    syn::Field {
        attrs: Vec::new(),
        vis: syn::Visibility::Inherited,
        ident: Some(deque_tail_ident(&field.ident)),
        colon_token: None,
        ty: parse_quote!(::std::sync::Arc<::std::sync::atomic::AtomicU64>),
        mutability: syn::FieldMutability::None,
    }
}

fn no_sync_phantom() -> syn::Field {
    syn::Field {
        attrs: Vec::new(),
        vis: syn::Visibility::Inherited,
        ident: Some(no_syn_phantom_ident()),
        colon_token: None,
        ty: parse_quote!(::std::marker::PhantomData<::std::sync::MutexGuard<'static, ()>>),
        mutability: syn::FieldMutability::None,
    }
}

impl From<&Model> for Struct {
    fn from(model: &Model) -> Self {
        use crate::model::Wrapper as W;

        let ty = match model.backend {
            Backend::Sled => parse_quote!(::dbstruct::sled::Tree),
            Backend::HashMap => parse_quote!(::dbstruct::stores::HashMap),
            Backend::BTreeMap => parse_quote!(::dbstruct::stores::BTreeMap),
            Backend::Trait { .. } => parse_quote!(DS),
            #[cfg(test)]
            Backend::Test => unreachable!("Test backend is not supported for codegen"),
        };

        let db = syn::Field {
            attrs: Vec::new(),
            vis: syn::Visibility::Inherited,
            ident: Some(syn::Ident::new("ds", proc_macro2::Span::call_site())),
            colon_token: None,
            ty,
            mutability: syn::FieldMutability::None,
        };

        let mut member_vars: Vec<_> = model
            .fields
            .iter()
            .flat_map(|field| match &field.wrapper {
                W::Vec { .. } => [vec_len_field(field)].to_vec(),
                W::VecDeque { .. } => [deque_head_field(field), deque_tail_field(field)].to_vec(),
                _ => Vec::new(),
            })
            .collect();

        member_vars.push(no_sync_phantom());

        Struct {
            ident: model.ident.clone(),
            vis: model.vis.clone(),
            member_vars,
            db,
        }
    }
}
