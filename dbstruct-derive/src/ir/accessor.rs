use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{parse_quote, parse_quote_spanned};

use crate::model::{Field, Wrapper};

use super::struct_def::{deque_head_ident, deque_tail_ident, vec_len_ident};

pub struct Accessor {
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub returns: syn::Type,
    pub body: syn::Block,
}

impl Accessor {
    pub fn from(field: Field, ds: syn::Type) -> Self {
        let key = field.key;
        let (body, returns) = match field.wrapper {
            #[allow(unused_variables)]
            Wrapper::Vec { ty } => {
                let len_ident = vec_len_ident(&field.ident);
                let body = parse_quote!({
                    dbstruct::wrapper::Vec::new(self.ds.clone(), #key, self.#len_ident.clone())
                });
                let returns = parse_quote_spanned!(ty.span()=> dbstruct::wrapper::Vec<#ty, #ds>);
                (body, returns)
            }
            Wrapper::VecDeque { ty } => {
                let head_ident = deque_head_ident(&field.ident);
                let tail_ident = deque_tail_ident(&field.ident);
                let body = parse_quote!({
                    dbstruct::wrapper::VecDeque::new(self.ds.clone(), #key, self.#head_ident.clone(), self.#tail_ident.clone())
                });
                let returns =
                    parse_quote_spanned!(ty.span()=> dbstruct::wrapper::VecDeque<#ty,#ds>);
                (body, returns)
            }
            #[allow(unused_variables)]
            Wrapper::Map { key_ty, val_ty } => {
                let body = parse_quote!({
                    dbstruct::wrapper::Map::new(self.ds.clone(), #key)
                });
                // Using proc_macro2 version until
                // https://github.com/rust-lang/rust/issues/54725 stabalizes
                let span = key_ty
                    .span()
                    .join(val_ty.span())
                    .unwrap_or(Span::call_site());
                let returns =
                    parse_quote_spanned!(span=> dbstruct::wrapper::Map<#key_ty, #val_ty, #ds>);
                (body, returns)
            }
            #[allow(unused_variables)]
            Wrapper::DefaultTrait { ty } => {
                let body = parse_quote!({
                    dbstruct::wrapper::DefaultTrait::new(self.ds.clone(), #key)
                });
                let returns =
                    parse_quote_spanned!(ty.span()=> dbstruct::wrapper::DefaultTrait<#ty, #ds>);
                (body, returns)
            }
            #[allow(unused_variables)]
            Wrapper::DefaultValue { ty, value } => {
                let body = parse_quote_spanned!(ty.span()=> {
                    let default_value = #value;
                    dbstruct::wrapper::DefaultValue::new(self.ds.clone(), #key, default_value)
                });
                let returns =
                    parse_quote_spanned!(ty.span()=> dbstruct::wrapper::DefaultValue<#ty, #ds>);
                (body, returns)
            }
            #[allow(unused_variables)]
            Wrapper::Option { ty } => {
                let body = parse_quote!({
                    dbstruct::wrapper::OptionValue::new(self.ds.clone(), #key)
                });
                let returns =
                    parse_quote_spanned!(ty.span()=> dbstruct::wrapper::OptionValue<#ty, #ds>);
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
    fn default() {
        let field = Field {
            ident: parse_quote!(test_a),
            vis: parse_quote!(pub),
            wrapper: Wrapper::DefaultTrait {
                ty: parse_quote!(u8),
            },
            key: 1,
        };
        let ds_ty = parse_quote!(DS);
        let _a = Accessor::from(field, ds_ty);
    }

    #[test]
    fn defaultval() {
        let field = Field {
            ident: parse_quote!(test_a),
            vis: parse_quote!(pub),
            wrapper: Wrapper::DefaultValue {
                ty: parse_quote!(u8),
                value: parse_quote!(5 + 12),
            },
            key: 1,
        };
        let ds_ty = parse_quote!(DS);
        let _a = Accessor::from(field, ds_ty);
    }

    #[test]
    fn option() {
        let field = Field {
            ident: parse_quote!(test_a),
            vis: parse_quote!(pub),
            wrapper: Wrapper::Option {
                ty: parse_quote!(u8),
            },
            key: 1,
        };
        let ds_ty = parse_quote!(DS);
        let _a = Accessor::from(field, ds_ty);
    }

    #[test]
    fn vec() {
        let field = Field {
            ident: parse_quote!(test_a),
            vis: parse_quote!(pub),
            wrapper: Wrapper::Vec {
                ty: parse_quote!(u8),
            },
            key: 1,
        };
        let ds_ty = parse_quote!(DS);
        let _a = Accessor::from(field, ds_ty);
    }

    #[test]
    fn map() {
        let field = Field {
            ident: parse_quote!(test_a),
            vis: parse_quote!(pub),
            wrapper: Wrapper::Map {
                key_ty: parse_quote!(u8),
                val_ty: parse_quote!(u16),
            },
            key: 1,
        };
        let ds_ty = parse_quote!(DS);
        let _a = Accessor::from(field, ds_ty);
    }
}
