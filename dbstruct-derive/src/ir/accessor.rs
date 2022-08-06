use syn::parse_quote;

use crate::model::{DbKey, Field, Wrapper};

pub struct Accessor {
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub returns: syn::Type,
    pub body: syn::Expr,
}

impl Accessor {
    pub fn from(field: Field, keys: &DbKey) -> Self {
        let wrappers = "dbstruct::wrappers";
        let key = keys.prefix(&field.ident);
        let (body, returns) = match field.wrapper {
            #[allow(unused_variables)]
            Wrapper::Vec { ty } => {
                let len_ident = format!("{}_len", field.ident);
                let body = parse_quote!(
                #wrappers::Vec::new(self.ds.clone(), #key, self.#len_ident.clone())
                );
                let returns = parse_quote!(#wrappers::Vec<#ty, DS>);
                (body, returns)
            }
            #[allow(unused_variables)]
            Wrapper::Map { key_ty, val_ty } => {
                let body = parse_quote!(
                #wrappers::Map::new(self.ds.clone(), #key)
                );
                let returns = parse_quote!(#wrappers::Map<#key_ty, #val_ty, DS>);
                (body, returns)
            },
            #[allow(unused_variables)]
            Wrapper::DefaultTrait { ty } => {
                let body = parse_quote!(
                #wrappers::DefaultTrait::new(self.ds.clone(), #key)
                );
                let returns = parse_quote!(#wrappers::DefaultTrait<#ty, DS>);
                (body, returns)
            },
            #[allow(unused_variables)]
            Wrapper::DefaultValue { ty, value } => {
                let body = parse_quote!(
                    let default_value = $value;
                    #wrappers::DefaultValue::new(self.ds.clone(), #key, default_value)
                );
                let returns = parse_quote!(#wrappers::DefaultValue<#ty, DS>);
                (body, returns)
            },
            #[allow(unused_variables)]
            Wrapper::Option { ty } => {
                let body = parse_quote!(
                #wrappers::OptionValue::new(self.ds.clone(), #key)
                );
                let returns = parse_quote!(#wrappers::OptionValue<#ty, DS>);
                (body, returns)
            },
        };

        Self {
            vis: field.vis,
            ident: field.ident,
            returns,
            body,
        }
    }
}
