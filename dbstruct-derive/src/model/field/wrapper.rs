use std::iter::Peekable;
use std::mem;

use proc_macro2::TokenTree;

mod errors;
use errors::GetSpan;
pub use errors::{Error, ErrorVariant};

#[derive(Debug, PartialEq, Eq)]
pub enum Wrapper {
    Vec { ty: syn::Type },
    Map { ty: syn::Type },
    DefaultTrait { ty: syn::Type },
    DefaultValue { ty: syn::Type, value: syn::Expr },
    Option { ty: syn::Type },
}

pub enum WrapperAttributes {
    DefaultTrait { span: proc_macro2::Span },
    DefaultValue { expr: syn::Expr },
}

fn is_relevant(att: &syn::Attribute) -> bool {
    let ident = match att.path.segments.first().map(|s| &s.ident) {
        Some(ident) => ident.to_string(),
        None => return false,
    };

    &ident == "dbstruct"
}

/* FIX: not a good function, look for a crate
 * that is tested replace before releasing the crate
 * * <01-08-22, dvdsk noreply@davidsk.dev> */
fn unescape_literal(s: &str) -> String {
    let s = s.trim_matches('"');
    let s = s.replace(r#"\"#, r#""#);
    s
}

fn parse(
    tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
) -> Result<WrapperAttributes, Error> {
    use ErrorVariant::*;
    let first_token = tokens
        .next()
        .expect("should only get here if peek returned Some");
    match first_token {
        TokenTree::Ident(ident) if ident.to_string() == "Default" => match tokens.peek() {
            None => {
                tokens.next();
                return Ok(WrapperAttributes::DefaultTrait { span: ident.span() });
            }
            Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => {
                tokens.next();
                return Ok(WrapperAttributes::DefaultTrait { span: ident.span() });
            }
            Some(TokenTree::Punct(punct)) if punct.as_char() == '=' => {
                let punct = punct.span();
                match tokens.nth(1) {
                None => return Err(MissingDefaultValue.with_span(punct)),
                Some(TokenTree::Literal(lit)) => {
                    let expr = lit.to_string();
                    let expr = unescape_literal(&expr);
                    let expr: syn::Expr = syn::parse_str(&expr)
                        .map_err(|err| ValueNotExpression(err).with_span(lit))?;
                    return Ok(WrapperAttributes::DefaultValue { expr });
                }
                Some(other) => return Err(InvalidDefaultArg.with_span(other)),
            }},
            Some(_other) => return Err(InvalidDefaultArg.with_span(_other)),
        },
        TokenTree::Ident(ident) => return Err(NotAWrapper(ident).has_span()),
        _ => return Err(InvalidSyntax(first_token).has_span()),
    }
}

fn as_wrapper(att: syn::Attribute) -> Result<Option<WrapperAttributes>, Error> {
    use ErrorVariant::*;
    let tokens = match att.tokens.into_iter().nth(0) {
        Some(tokens) => tokens,
        None => return Ok(None),
    };

    let tokens = match tokens {
        TokenTree::Group(group) => group.stream(),
        _other => return Err(InvalidTokenTree.with_span(_other)),
    };

    let mut res = None;
    let mut tokens = dbg!(tokens).into_iter().peekable();
    while let Some(token) = tokens.peek() {
        if res.is_none() {
            res = Some(parse(&mut tokens)?);
        } else {
            return Err(MultipleWrapperAttributes.with_span(token));
        }
    }
    Ok(res)
}

impl Wrapper {
    /// takes relevant attributes from `attributes` and determines the wrapper
    pub fn try_from(attributes: &mut Vec<syn::Attribute>, ty: syn::Type) -> Result<Self, Error> {
        use ErrorVariant::*;
        use WrapperAttributes::*;

        let (mut relevant, other): (Vec<_>, Vec<_>) =
            mem::take(attributes).into_iter().partition(is_relevant);
        *attributes = other; /* TODO: use drain_filter when it stabilizes <31-07-22> */

        let attribute = relevant.pop().map(as_wrapper).transpose()?.flatten();
        if let Some(other) = relevant.pop() {
            return Err(MultipleAttributes.with_span(&other));
        }

        Ok(match (outer_type(&ty)?.as_str(), attribute) {
            ("Vec", None) => Self::Vec { ty },
            ("Vec", Some(_)) => todo!("Vec with an attribute"),
            ("Option", None) => Self::Option { ty },
            // in the future use proc_macro2::span::join() to give an
            // error at the type and the default trait attribute
            ("Option", Some(DefaultTrait { span })) => return Err(OptionNotAllowed.with_span(span)),
            ("Option", Some(_)) => todo!("Option with default value"),
            ("HashMap", None) => Self::Map { ty },
            ("HashMap", Some(_)) => todo!("Hashmap with an attribute"),
            (_, None) => return Err(NoDefaultType.with_span(ty)),
            (_, Some(DefaultTrait { .. })) => Self::DefaultTrait { ty },
            (_, Some(DefaultValue { expr })) => Self::DefaultValue { ty, value: expr },
        })
    }
}

fn outer_type(type_path: &syn::Type) -> Result<String, Error> {
    use ErrorVariant::EmptyTypeForbidden;
    match type_path {
        syn::Type::Path(syn::TypePath { path, .. }) => Ok(path
            .segments
            .iter()
            .next()
            .ok_or(EmptyTypeForbidden.with_span(type_path))?
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
                syn::parse_quote!(#[dbstruct(Default="5u8")]),
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
                syn::parse_quote!(#[dbstruct(Default="format!(\"hello, {}\", 5u8)")]),
                syn::parse_quote!(#[b]),
            ];
            let ty_u8: syn::Type = syn::parse_quote!(u8);
            let wrapper = Wrapper::try_from(&mut attributes.to_vec(), ty_u8.clone()).unwrap();
            let value: syn::Expr = syn::parse_quote!(format!("hello, {}", 5u8));
            assert_eq!(wrapper, Wrapper::DefaultValue { ty: ty_u8, value })
        }
    }
}
