use proc_macro2::Delimiter::Parenthesis;
use proc_macro2::TokenStream;
use proc_macro2::TokenTree::{Group, Ident};

use quote::{quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{AttrStyle, Attribute, Expr, Field, Type, TypePath};

use crate::key::DbKey;

mod default;
mod hashmap;
mod vec_by_idx;

fn outer_type(type_path: &Type) -> Result<String, ()> {
    match type_path {
        Type::Path(TypePath { path, .. }) => {
            Ok(path.segments.iter().next().ok_or(())?.ident.to_string())
        }
        _ => todo!("handle other types"), // TODO handle edgecases
    }
}

fn attribute<'a>(attrs: &'a [Attribute]) -> Result<(Option<String>, Option<Expr>), ()> {
    if attrs.len() > 1 {
        todo!("for now we can only have one attribute, handle this as error")
    }

    let attr = attrs.first().ok_or(())?;
    if attr.style != AttrStyle::Outer {
        todo!("Error handling for attr.style")
    }

    let path = attr
        .path
        .segments
        .iter()
        .next()
        .ok_or(())?
        .ident
        .to_string();
    if path != "dbstruct" {
        todo!("unknown attribute do not handle")
    }

    let mut attr_option = None;
    let mut attr_value = None;
    if attr.tokens.is_empty() {
        return Ok((attr_option, attr_value));
    }

    let mut trees = attr.tokens.clone().into_iter();
    let mut group = match trees.next() {
        Some(Group(g)) => g.stream().into_iter(),
        _ => panic!(),
    };

    if let Some(Ident(ident)) = group.next() {
        attr_option = Some(ident.to_string());
    }

    if let Some(Group(g)) = group.next() {
        assert_eq!(g.delimiter(), Parenthesis);

        let test: syn::LitStr =
            syn::parse2(g.stream()).expect("todo error handling: not a string literal");
        let expr: syn::Expr = syn::parse_str(&test.value()).unwrap();
        attr_value = Some(expr);
    }

    return Ok((attr_option, attr_value));
}

enum Interface {
    DefaultValue(Expr),
    DefaultTrait,
    Option,
    Vec,
    IndexTree { value_type: Type },
    HashMap,
    PrefixTree { key_type: Type, value_type: Type },
}

fn unwrap_type_path(path: &Type) -> Option<&syn::Path> {
    match path {
        Type::Path(TypePath { path, .. }) => Some(path),
        _ => return None,
    }
}

fn vec_by_idx(field: &Field) -> Result<Interface, ()> {
    use syn::GenericArgument;
    use syn::PathArguments;

    let type_path = unwrap_type_path(&field.ty).unwrap();
    let generic = match &type_path.segments.first().ok_or(())?.arguments {
        PathArguments::AngleBracketed(args) => args,
        _ => return Err(()),
    };

    let r#type = match generic.args.first().ok_or(())? {
        GenericArgument::Type(r#type) => r#type,
        _ => return Err(()),
    };
    Ok(Interface::IndexTree {
        value_type: r#type.clone(),
    })
}

fn prefix_tree(field: &Field) -> Result<Interface, ()> {
    use syn::GenericArgument;
    use syn::PathArguments;

    let type_path = unwrap_type_path(&field.ty).unwrap();
    let generic = match &type_path.segments.first().ok_or(())?.arguments {
        PathArguments::AngleBracketed(args) => args,
        _ => return Err(()),
    };

    let mut types = generic.args.iter();
    let key_type = match types.next().ok_or(())? {
        GenericArgument::Type(r#type) => r#type,
        _ => return Err(()),
    }
    .clone();
    let value_type = match types.next().ok_or(())? {
        GenericArgument::Type(r#type) => r#type,
        _ => return Err(()),
    }
    .clone();
    Ok(Interface::PrefixTree {
        key_type,
        value_type,
    })
}

impl TryFrom<&Field> for Interface {
    type Error = ();
    fn try_from(field: &Field) -> Result<Self, Self::Error> {
        let outer = outer_type(&field.ty)?;
        let outer = outer.as_str();
        let (attr, value) = attribute(&field.attrs).unwrap_or((None, None));
        let attr = attr.as_ref().map(String::as_str);

        match (outer, attr, &value) {
            ("Option", None, None) => Ok(Self::Option),
            ("Vec", None, None) => Ok(vec_by_idx(field).unwrap()),
            ("Vec", Some("no_idx"), None) => Ok(Self::Vec),
            ("HashMap", None, None) => Ok(prefix_tree(field).unwrap()),
            ("HashMap", Some("no_prefix"), None) => Ok(Self::HashMap),
            (_, Some("default"), None) => Ok(Self::DefaultTrait),
            (_, Some("default"), Some(val)) => Ok(Self::DefaultValue(val.clone())),
            // TODO error handling w a proper span
            _ => todo!(
                "Did not match any case: \"{outer}, {attr:?}, {value:?}\", probably not supported"
            ),
        }
    }
}

pub fn generate((field, keys): (&Field, &DbKey)) -> TokenStream {
    let ident = field.ident.as_ref().unwrap();
    let full_type = &field.ty;
    let key = ident.to_string();
    let outer_type = match Interface::try_from(field) {
        Ok(info) => info,
        Err(..) => {
            let span = field.span();
            return quote_spanned!(span=> compile_error!("[dbstruct] Every type except vector 
                                                         must be contained in an Option"););
        }
    };

    match outer_type {
        Interface::Option => {
            let span = full_type.span();
            let default_val = quote_spanned!(span=> Option::None);
            default::methods(ident, full_type, &key, &default_val)
        }
        Interface::Vec => {
            let span = full_type.span();
            let default_val = quote_spanned!(span=> Vec::new);
            default::methods(ident, full_type, &key, &default_val)
        }
        Interface::IndexTree { value_type } => {
            let fn_idx_key = keys.fn_idx_key(ident);
            let expr_prefix = keys.expr_prefix(ident);
            vec_by_idx::methods(ident, &value_type, fn_idx_key, expr_prefix)
        }
        Interface::DefaultTrait => {
            let span = full_type.span();
            let default_val = quote_spanned!(span=> Default::default());
            default::methods(ident, full_type, &key, &default_val)
        }
        Interface::DefaultValue(default_val) => {
            let default_val = default_val.to_token_stream();
            default::methods(ident, full_type, &key, &default_val)
        }

        Interface::HashMap => {
            let span = full_type.span();
            let default_val = quote_spanned!(span=> std::collections::HashMap::new);
            default::methods(ident, full_type, &key, &default_val)
        }
        Interface::PrefixTree {
            key_type,
            value_type,
        } => {
            let expr_prefix = keys.expr_prefix(ident);
            hashmap::methods(ident, &key_type, &value_type, expr_prefix)
        }
    }
}
