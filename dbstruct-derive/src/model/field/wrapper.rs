use std::iter::Peekable;
use std::mem;

use proc_macro2::TokenTree;

mod errors;
pub use errors::{Error, ErrorVariant};

#[derive(Debug, PartialEq, Eq)]
pub enum Wrapper {
    Vec {
        ty: syn::Type,
    },
    Map {
        key_ty: syn::Type,
        val_ty: syn::Type,
    },
    DefaultTrait {
        ty: syn::Type,
    },
    DefaultValue {
        ty: syn::Type,
        value: syn::Expr,
    },
    Option {
        ty: syn::Type,
    },
}

#[derive(Debug)]
pub enum Attribute {
    DefaultTrait { span: proc_macro2::Span },
    DefaultValue { expr: syn::Expr },
    // NoWrap { span: proc_macro2::Span },
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

fn parse_default(
    span: proc_macro2::Span,
    tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
) -> Result<Attribute, Error> {
    use ErrorVariant::*;
    match tokens.peek() {
        None => {
            tokens.next();
            return Ok(Attribute::DefaultTrait { span });
        }
        Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => {
            tokens.next();
            return Ok(Attribute::DefaultTrait { span });
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
                    return Ok(Attribute::DefaultValue { expr });
                }
                Some(other) => return Err(InvalidDefaultArg.with_span(other)),
            }
        }
        Some(_other) => return Err(InvalidDefaultArg.with_span(_other)),
    }
}

fn parse(tokens: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Result<Attribute, Error> {
    use ErrorVariant::*;
    let first_token = tokens
        .next()
        .expect("should only get here if peek returned Some");
    match first_token {
        TokenTree::Ident(ident) if ident.to_string() == "Default" => {
            parse_default(ident.span(), tokens)
        }
        TokenTree::Ident(ident) => return Err(NotAWrapper(ident).has_span()),
        _ => return Err(InvalidSyntax(first_token).has_span()),
    }
}

fn as_wrapper(att: syn::Attribute) -> Result<Option<Attribute>, Error> {
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
    let mut tokens = tokens.into_iter().peekable();
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
        use Attribute::*;
        use ErrorVariant::*;

        let (mut relevant, other): (Vec<_>, Vec<_>) =
            mem::take(attributes).into_iter().partition(is_relevant);
        *attributes = other; /* TODO: use drain_filter when it stabilizes <31-07-22> */

        let attribute = relevant.pop().map(as_wrapper).transpose()?.flatten();
        if let Some(other) = relevant.pop() {
            return Err(MultipleAttributes.with_span(&other));
        }

        Ok(match (outer_type(&ty)?.as_str(), attribute) {
            ("Vec", None) => Self::Vec { ty: inner_type(&ty, "Vec")? },
            ("Option", None) => Self::Option { ty: inner_type(&ty, "Option")? },
            // in the future use proc_macro2::span::join() to give an
            // error at the type and the default trait attribute
            ("Option", Some(DefaultTrait { span })) => return Err(OptionNotAllowed.with_span(span)),
            ("HashMap", None) => {
                let (key_ty, val_ty) = hashmap_types(&ty)?;
                Self::Map { key_ty, val_ty }
            }
            (_, None) => return Err(NoDefaultType.with_span(ty)),
            (_, Some(DefaultTrait { .. })) => Self::DefaultTrait { ty },
            (_, Some(DefaultValue { expr })) => Self::DefaultValue { ty, value: expr },
        })
    }
}

fn inner_type(ty: &syn::Type, outer_ty: &'static str) -> Result<syn::Type, Error> {
    let mut generics = generic_types(ty)?;
    let ty = generics
        .next()
        .ok_or(
            ErrorVariant::TooFewGenerics {
                ty: outer_ty,
                n_needed: 1,
            }
            .with_span(ty),
        )??
        .clone();

    if let Some(other_generic) = generics.next() {
        return Err(ErrorVariant::TooManyGenerics {
            ty: outer_ty,
            n_needed: 1,
        }
        .with_span(other_generic?));
    }

    Ok(ty)
}

fn hashmap_types(ty: &syn::Type) -> Result<(syn::Type, syn::Type), Error> {
    let mut generics = generic_types(ty)?;
    let key_ty = generics
        .next()
        .ok_or(
            ErrorVariant::TooFewGenerics {
                ty: "HashMap",
                n_needed: 2,
            }
            .with_span(ty),
        )??
        .clone();
    let val_ty = generics
        .next()
        .ok_or(
            ErrorVariant::TooFewGenerics {
                ty: "HashMap",
                n_needed: 2,
            }
            .with_span(ty),
        )??
        .clone();

    if let Some(other_generic) = generics.next() {
        return Err(ErrorVariant::TooManyGenerics {
            ty: "HashMap",
            n_needed: 2,
        }
        .with_span(other_generic?));
    }

    Ok((key_ty, val_ty))
}

fn generic_types(ty: &syn::Type) -> Result<impl Iterator<Item = Result<&syn::Type, Error>>, Error> {
    let punctuated = match ty {
        syn::Type::Path(syn::TypePath {
            path: syn::Path { segments, .. },
            ..
        }) => segments,
        _ => unreachable!("should only run in match arm when matching HashMap"),
    };

    // aliasing HashMap can result in a macro panic. The person making
    // the alias will probably understand the panic message
    let arguments = &punctuated
        .first()
        .expect("already checked in `outer_type` function")
        .arguments;

    let types = match arguments {
        syn::PathArguments::AngleBracketed(bracketed) => bracketed.args.iter(),
        _ => unreachable!("types in named structs can only have angle bracketed type arguments"),
    }
    .map(move |generic| match generic {
        syn::GenericArgument::Type(ty) => Ok(ty),
        _ => Err(ErrorVariant::NotATypeGeneric.with_span(ty)),
    });

    Ok(types)
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
    use syn::parse_quote;

    mod default_trait {
        use super::*;

        #[test]
        fn u8() {
            let attributes: &[syn::Attribute] =
                &[parse_quote!(#[dbstruct(Default)]), parse_quote!(#[b])];
            let ty_u8: syn::Type = parse_quote!(u8);
            let wrapper = Wrapper::try_from(&mut attributes.to_vec(), ty_u8.clone()).unwrap();
            assert_eq!(wrapper, Wrapper::DefaultTrait { ty: ty_u8 })
        }

        #[test]
        fn vec() {
            let attributes: &[syn::Attribute] =
                &[parse_quote!(#[dbstruct(Default)]), parse_quote!(#[b])];
            let field_ty: syn::Type = parse_quote!(Vec<u8>);
            let wrapper = Wrapper::try_from(&mut attributes.to_vec(), field_ty.clone()).unwrap();
            assert_eq!(wrapper, Wrapper::DefaultTrait { ty: field_ty })
        }
    }

    #[test]
    fn vec() {
        let inner_ty: syn::Type = parse_quote!(u32);
        let ty_vec: syn::Type = parse_quote!(Vec<u32>);
        let wrapper = Wrapper::try_from(&mut Vec::new(), ty_vec.clone()).unwrap();
        assert_eq!(wrapper, Wrapper::Vec { ty: inner_ty })
    }

    #[test]
    fn map() {
        let key_ty: syn::Type = parse_quote!(u8);
        let val_ty: syn::Type = parse_quote!(Vec<u16>);
        let ty_hashmap: syn::Type = parse_quote!(HashMap<u8, Vec<u16>>);
        let wrapper = Wrapper::try_from(&mut Vec::new(), ty_hashmap.clone()).unwrap();
        assert_eq!(wrapper, Wrapper::Map { key_ty, val_ty })
    }

    #[test]
    fn option() {
        let inner_ty: syn::Type = parse_quote!(u16);
        let ty: syn::Type = parse_quote!(Option<u16>);
        let wrapper = Wrapper::try_from(&mut Vec::new(), ty).unwrap();
        assert_eq!(wrapper, Wrapper::Option { ty: inner_ty })
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
