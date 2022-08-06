use syn::parse_quote;

use crate::model::{Field, Model};

pub struct Struct {
    pub ident: syn::Ident,
    pub vis: syn::Visibility,
    pub vars: Vec<syn::Field>,
}

fn ds_field() -> syn::Field {
    syn::Field {
        attrs: Vec::new(),
        vis: syn::Visibility::Inherited,
        ident: Some(syn::Ident::new("ds", proc_macro2::Span::call_site())),
        colon_token: None,
        ty: parse_quote!(DS),
    }
}

fn as_len_field(field: &Field) -> syn::Field {
    let name = format!("{}_len", field.ident);
    syn::Field {
        attrs: Vec::new(),
        vis: syn::Visibility::Inherited,
        ident: Some(syn::Ident::new(&name, proc_macro2::Span::call_site())),
        colon_token: None,
        ty: parse_quote!(std::sync::Arc<std::sync::atomic::AtomicUsize>),
    }
}

impl From<&Model> for Struct {
    fn from(model: &Model) -> Self {
        let vars = model
            .fields
            .iter()
            .filter(|f| f.is_vec())
            .map(as_len_field)
            .chain(std::iter::once(ds_field()))
            .collect();
        Struct {
            ident: model.ident.clone(),
            vis: model.vis.clone(),
            vars,
        }
    }
}
