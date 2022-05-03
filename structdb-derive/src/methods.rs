use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Attribute, Field, Ident, PathSegment, Type, TypePath};

mod option;
mod vec;

fn outer_type(type_path: &Type) -> Result<String, ()> {
    match type_path {
        Type::Path(TypePath { path, .. }) => {
            Ok(path.segments.iter().next().ok_or(())?.ident.to_string())
        }
        _ => todo!("handle other types"), // TODO handle edgecases
    }
}

fn attribute<'a>(attrs: &'a [Attribute]) -> Result<Option<String>, ()> {
    dbg!(attrs);
    if attrs.len() > 1 {
        todo!("handle this as error")
    }

    // Ok(attrs.first(0).map)
    todo!()
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
        let attr = attribute(&field.attrs)?;
        match (outer.as_str(), attr.as_ref().map(String::as_str)) {
            ("Option", None) => Ok(Self::Option),
            ("Vec", None) => Ok(Self::Vec),
            ("Vec", Some("indexed")) => Ok(Self::VecByIdx),
            _ => todo!("nice errors"),
        }
    }
}

pub fn generate(field: &Field) -> TokenStream {
    let ident = field.ident.as_ref().unwrap();
    let full_type = &field.ty;
    let key = ident.to_string();
    let outer_type = match Interface::try_from(field) {
        Ok(info) => info,
        Err(..) => {
            let span = field.span();
            return quote_spanned!(span=> compile_error!("[structdb] Every type except vector 
                                                        must be contained in an Option"););
        }
    };

    match outer_type {
        Interface::Option => {
            let setter = option::setter(ident, full_type, &key);
            let getter = option::getter(ident, full_type, &key);
            let update = option::update(ident, full_type, &key);
            let compare_and_swap = option::compare_and_swap(ident, full_type, &key);

            quote!(
                #setter
                #getter
                #update
                #compare_and_swap
            )
        }
        Interface::Vec => {
            let setter = vec::setter(ident, full_type, &key);
            let getter = vec::getter(ident, full_type, &key);
            let update = vec::update(ident, full_type, &key);
            let compare_and_swap = vec::compare_and_swap(ident, full_type, &key);

            quote!(
                #setter
                #getter
                #update
                #compare_and_swap
            )
        }
        _ => todo!("implement other interfaces"),
    }
}
