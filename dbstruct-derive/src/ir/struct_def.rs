use syn::parse_quote;

use crate::model::backend::Backend;
use crate::model::{Field, Model};

pub struct Struct {
    pub ident: syn::Ident,
    pub vis: syn::Visibility,
    /// extra variables such as the current length
    /// of the vector wrapper
    pub len_vars: Vec<syn::Field>,
    pub db: syn::Field,
}

pub fn as_len_ident(ident: &syn::Ident) -> syn::Ident {
    let name = format!("{}_len", ident);
    syn::Ident::new(&name, proc_macro2::Span::call_site())
}

fn as_len_field(field: &Field) -> syn::Field {
    syn::Field {
        attrs: Vec::new(),
        vis: syn::Visibility::Inherited,
        ident: Some(as_len_ident(&field.ident)),
        colon_token: None,
        ty: parse_quote!(std::sync::Arc<std::sync::atomic::AtomicUsize>),
    }
}

impl From<&Model> for Struct {
    fn from(model: &Model) -> Self {
        let len_vars = model
            .fields
            .iter()
            .filter(|f| f.is_vec())
            .map(as_len_field)
            .collect();

        let ty = match model.backend {
            Backend::Sled => parse_quote!(::dbstruct::sled::Tree),
            Backend::HashMap => parse_quote!(::dbstruct::stores::HashMap),
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
        };

        Struct {
            ident: model.ident.clone(),
            vis: model.vis.clone(),
            len_vars,
            db,
        }
    }
}
