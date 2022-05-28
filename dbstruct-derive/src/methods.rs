use proc_macro2::TokenStream;

use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::{AttrStyle, Attribute, Field, Type, TypePath};

use crate::key::DbKey;

mod default;
mod vec_by_idx;

fn outer_type(type_path: &Type) -> Result<String, ()> {
    match type_path {
        Type::Path(TypePath { path, .. }) => {
            Ok(path.segments.iter().next().ok_or(())?.ident.to_string())
        }
        _ => todo!("handle other types"), // TODO handle edgecases
    }
}

fn attribute<'a>(attrs: &'a [Attribute]) -> Result<(Option<String>, Option<String>), ()> {
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

    use proc_macro2::TokenTree::{Ident, Group};
    use proc_macro2::Delimiter::Parenthesis; 

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

        let test: syn::LitStr = syn::parse2(g.stream()).unwrap();
        attr_value = Some(test.value());
    }

    return Ok((attr_option, attr_value));
}

enum Interface {
    DefaultValue(String),
    DefaultTrait,
    Option,
    Vec,
    VecByIdx,
    PrefixTree,
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
            ("Vec", None, None) => Ok(Self::VecByIdx),
            ("Vec", Some("no_idx"), None) => Ok(Self::Vec),
            ("HashMap", None, None) => Ok(Self::PrefixTree),
            (_, Some("default"), None) => Ok(Self::DefaultTrait),
            (_, Some("default"), Some(val)) => Ok(Self::DefaultValue(val.clone())),
            _ => todo!("Did not match any case: \"{outer}, {attr:?}, {value:?}\""),
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
            let default_val = quote_spanned!(span=> Vec::new());
            default::methods(ident, full_type, &key, &default_val)
        }
        Interface::DefaultTrait => {
            let span = full_type.span();
            let default_val = quote_spanned!(span=> Default::default());
            default::methods(ident, full_type, &key, &default_val)
        }
        Interface::DefaultValue(unpared_value) => {
            // TODO error handling
            let default_val = unpared_value
                .parse()
                .expect("could not parse default value to rust expression");
            default::methods(ident, full_type, &key, &default_val)
        }

        Interface::VecByIdx => {
            let key_gen = keys.idx_key_method(ident);
            vec_by_idx::methods(ident, full_type, key_gen)
        }
        Interface::PrefixTree => {
            todo!()
        }
    }
}
