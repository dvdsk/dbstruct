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
        let key = keys.prefix(&field.ident);
        let (body, returns) = match field.wrapper {
            #[allow(unused_variables)]
            Wrapper::Vec { ty } => {
                let len_ident = format!("{}_len", field.ident);
                let body = parse_quote!(
                dbstruct::wrappers::Vec::new(self.ds.clone(), #key, self.#len_ident.clone())
                );
                let returns = parse_quote!(dbstruct::wrappers::Vec<#ty, DS>);
                (body, returns)
            }
            #[allow(unused_variables)]
            Wrapper::Map { key_ty, val_ty } => {
                let body = parse_quote!(
                dbstruct::wrappers::Map::new(self.ds.clone(), #key)
                );
                let returns = parse_quote!(dbstruct::wrappers::Map<#key_ty, #val_ty, DS>);
                (body, returns)
            }
            #[allow(unused_variables)]
            Wrapper::DefaultTrait { ty } => {
                let body = parse_quote!(
                dbstruct::wrappers::DefaultTrait::new(self.ds.clone(), #key)
                );
                let returns = parse_quote!(dbstruct::wrappers::DefaultTrait<#ty, DS>);
                (body, returns)
            }
            #[allow(unused_variables)]
            Wrapper::DefaultValue { ty, value } => {
                let body = parse_quote!(
                    let default_value = $value;
                    dbstruct::wrappers::DefaultValue::new(self.ds.clone(), #key, default_value)
                );
                let returns = parse_quote!(dbstruct::wrappers::DefaultValue<#ty, DS>);
                (body, returns)
            }
            #[allow(unused_variables)]
            Wrapper::Option { ty } => {
                let body = parse_quote!(
                dbstruct::wrappers::OptionValue::new(self.ds.clone(), #key)
                );
                let returns = parse_quote!(dbstruct::wrappers::OptionValue<#ty, DS>);
                (body, returns)
            }
        };

        Self {
            vis: field.vis,
            ident: field.ident,
            returns,
            body,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_accessor() {
        let field = Field {
            ident: parse_quote!(test_a),
            vis: parse_quote!(pub),
            wrapper: Wrapper::DefaultTrait {
                ty: parse_quote!(u8),
            },
        };
        let keys = DbKey::mock();
        let _a = Accessor::from(field, &keys);
    }
}
