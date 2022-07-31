use std::collections::HashSet;
use std::iter::Peekable;
use std::mem;

use proc_macro2::TokenTree;
use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::{parse_quote, Expr, Token};

#[derive(Debug, PartialEq, Eq)]
pub enum Wrapper {
    Vec { ty: syn::Type },
    Map { ty: syn::Type },
    DefaultTrait { ty: syn::Type },
    DefaultValue { ty: syn::Type, value: Expr },
    Option { ty: syn::Type },
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Can only have a single dbstruct attribute on a struct field")]
    MultipleAttributes,
    #[error("Invalid token tree expected Group")]
    InvalidTokenTree,
    #[error("Not a dbstruct wrapper annotation (try DefaultTrait or DefaultValue)")]
    NotAWrapper(syn::Ident),
    #[error("Not valid syntax for a dbstruct attribute, expected a single word as option")]
    InvalidSyntax(proc_macro2::TokenTree),
    #[error("Each field can have a maximum of two wrapper attributes")]
    MultipleWrapperAttributes,
    #[error("Option is already initialized as None, suggestion: remove DefaultTrait")]
    OptionNotAllowed,
    #[error("The empty type is not allowed")]
    EmptyTypeForbidden,
    #[error(
        "Every member needs a default value, annotate this to use a trait 
            or a fixed expression to generate one. Or use Option, Vec or HashMap"
    )]
    NoDefaultType,
    #[error("Invalid syntax: missing an expression for the default value")]
    MissingDefaultValue,
    #[error("Default value is not an expression")]
    ValueNotExpression(syn::parse::Error),
    #[error("Invalid argument for the Default attribute")]
    InvalidDefaultArg,
}

pub enum WrapperAttributes {
    DefaultTrait,
    DefaultValue { expr: syn::Expr },
}

fn is_relevant(att: &syn::Attribute) -> bool {
    let ident = match att.path.segments.first().map(|s| &s.ident) {
        Some(ident) => ident.to_string(),
        None => return false,
    };

    &ident == "dbstruct"
}

fn parse(
    tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
) -> Result<WrapperAttributes, Error> {
    let first_token = tokens
        .next()
        .expect("should only get here if peek returned Some");
    match first_token {
        TokenTree::Ident(ident) if ident.to_string() == "Default" => match dbg!(tokens.peek()) {
            None => {
                tokens.next();
                return Ok(WrapperAttributes::DefaultTrait);
            }
            Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => {
                tokens.next();
                return Ok(WrapperAttributes::DefaultTrait);
            }
            Some(TokenTree::Punct(punct)) if punct.as_char() == '=' => {
                let expr = tokens.nth(1).ok_or(Error::MissingDefaultValue)?;
                let expr: syn::Expr =
                    syn::parse2(expr.to_token_stream()).map_err(Error::ValueNotExpression)?;
                return Ok(WrapperAttributes::DefaultValue { expr });
            }
            _ => return Err(Error::InvalidDefaultArg),
        },
        _ => return Err(Error::InvalidSyntax(first_token)),
    }
}

fn as_wrapper(att: syn::Attribute) -> Result<Option<WrapperAttributes>, Error> {
    let tokens = match att.tokens.into_iter().nth(0) {
        Some(tokens) => tokens,
        None => return Ok(None),
    };

    let tokens = match tokens {
        TokenTree::Group(group) => group.stream(),
        _ => return Err(Error::InvalidTokenTree),
    };

    let mut res = None;
    let mut tokens = dbg!(tokens).into_iter().peekable();
    while let Some(_) = tokens.peek() {
        if res.is_none() {
            res = Some(parse(&mut tokens)?);
        } else {
            return Err(Error::MultipleWrapperAttributes);
        }
    }
    Ok(res)
}

impl Wrapper {
    /// takes relevant attributes from `attributes` and determines the wrapper
    pub fn try_from(attributes: &mut Vec<syn::Attribute>, ty: syn::Type) -> Result<Self, Error> {
        use WrapperAttributes::*;

        let (mut relevant, other): (Vec<_>, Vec<_>) =
            mem::take(attributes).into_iter().partition(is_relevant);
        *attributes = other; /* TODO: use drain_filter when it stabilizes <31-07-22> */

        let attribute = relevant.pop().map(as_wrapper).transpose()?.flatten();
        if !relevant.is_empty() {
            return Err(Error::MultipleAttributes);
        }

        Ok(match (outer_type(&ty)?.as_str(), attribute) {
            ("Vec", None) => Self::Vec { ty },
            ("Vec", Some(_)) => todo!("Vec with an attribute"),
            ("Option", None) => Self::Option { ty },
            ("Option", Some(DefaultTrait)) => return Err(Error::OptionNotAllowed),
            ("Option", Some(_)) => todo!("Option with default value"),
            ("HashMap", None) => Self::Map { ty },
            ("HashMap", Some(_)) => todo!("Hashmap with an attribute"),
            (_, None) => return Err(Error::NoDefaultType),
            (_, Some(DefaultTrait)) => Self::DefaultTrait { ty },
            (_, Some(DefaultValue { expr })) => Self::DefaultValue { ty, value: expr },
        })

        // match attribute {
        //     Some(DefaultTrait) => {
        //         if outer_type(&ty)? == "Option" {
        //             return Err(Error::OptionNotAllowed);
        //         } else {
        //             return Ok(Self::DefaultTrait { ty });
        //         }
        //     }
        //     Some(DefaultValue) => return Ok(Self::DefaultValue { ty }),
        //     None => (),
        // }
    }
}

fn outer_type(type_path: &syn::Type) -> Result<String, Error> {
    match type_path {
        syn::Type::Path(syn::TypePath { path, .. }) => Ok(path
            .segments
            .iter()
            .next()
            .ok_or(Error::EmptyTypeForbidden)?
            .ident
            .to_string()),
        _ => unreachable!("None path types probably do not occur in structs"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_trait() {
        let attributes: &[syn::Attribute] = &[
            syn::parse_quote!(#[dbstruct(Default)]),
            syn::parse_quote!(#[b]),
        ];
        let ty_u8: syn::Type = syn::parse_quote!(u8);
        let wrapper = Wrapper::try_from(&mut attributes.to_vec(), ty_u8.clone()).unwrap();
        assert_eq!(wrapper, Wrapper::DefaultTrait { ty: ty_u8 })
    }

    mod default_value {
        use super::*;

        #[test]
        fn literal_expression() {
            let attributes: &[syn::Attribute] = &[
                syn::parse_quote!(#[dbstruct(Default=5u8)]),
                syn::parse_quote!(#[b]),
            ];
            let ty_u8: syn::Type = syn::parse_quote!(u8);
            let wrapper = Wrapper::try_from(&mut attributes.to_vec(), ty_u8.clone()).unwrap();
            let value: syn::Expr = syn::parse_quote!(5u8);
            assert_eq!(wrapper, Wrapper::DefaultValue { ty: ty_u8, value })
        }

        #[test]
        fn function_call() {
            let attributes: &[syn::Attribute] = &[
                syn::parse_quote!(#[dbstruct(Default=format!("hello, {}", 5u8))]),
                syn::parse_quote!(#[b]),
            ];
            let ty_u8: syn::Type = syn::parse_quote!(u8);
            let wrapper = Wrapper::try_from(&mut attributes.to_vec(), ty_u8.clone()).unwrap();
            let value: syn::Expr = syn::parse_quote!(5u8);
            assert_eq!(wrapper, Wrapper::DefaultValue { ty: ty_u8, value })
        }
    }

}
