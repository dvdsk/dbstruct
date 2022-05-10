use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{AttrStyle, Attribute, Field, Type, TypePath};

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

fn attribute<'a>(attrs: &'a [Attribute]) -> Result<(Option<String>, Option<String>), ()> {
    dbg!(attrs);
    if attrs.len() > 1 {
        todo!("for now we can only have one attribute, handle this as error")
    }

    let attr = attrs.first().ok_or(())?;
    if attr.style != AttrStyle::Outer {
        todo!("Error handling for attr.style")
    }

    let path = attr.path.segments.iter().next().ok_or(())?.ident.to_string();
    if path != "dbstruct" {
        todo!("unknown attribute do not handle")
    }

    let token = (!attr.tokens.is_empty()).then(|| attr.tokens.to_string());
    Ok((, )) // TODO split token into Some(default) and Some(default_value_string)
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
        let (path, tokens) = attribute(&field.attrs).unwrap_or((None, None));
        let path = path.as_ref().map(String::as_str);

        match (outer, path, &tokens) {
            ("Option", None, None) => Ok(Self::Option),
            ("Vec", None, None) => Ok(Self::VecByIdx),
            ("Vec", Some("no_idx"), None) => Ok(Self::Vec),
            ("HashMap", None, None) => Ok(Self::PrefixTree),
            (_, Some("default"), None) => Ok(Self::DefaultTrait),
            (_, Some("default"), Some(val)) => Ok(Self::DefaultValue(val.clone())),
            _ => todo!("Did not match any case: \"{outer}, {path:?}, {tokens:?}\""),
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
            return quote_spanned!(span=> compile_error!("[dbstruct] Every type except vector 
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
            // todo use the option functions with a default val?
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
