mod errors;
pub use errors::{Error, ErrorVariant};
use proc_macro2::Span;

use std::iter::Peekable;

use proc_macro2::TokenTree;

#[derive(Debug, Clone, Copy)]
pub enum BackendOptionVariant {
    Sled,
    HashMap,
    BTreeMap,
    Trait,
    #[cfg(test)]
    Test,
}

#[derive(Debug, Clone, Copy)]
pub struct BackendOption {
    pub backend: BackendOptionVariant,
    pub span: Span,
}

#[derive(Debug)]
pub enum Options {
    Backend(BackendOption),
    Async,
}

/// attrs is the TokenStream returned by Attribute::parse_args();
pub fn parse(attrs: proc_macro2::TokenStream) -> Result<Vec<Options>, Error> {
    let mut attributes = Vec::new();
    let mut tokens = attrs.into_iter().peekable();
    while tokens.peek().is_some() {
        let attribute = parse_item(&mut tokens)?;
        attributes.push(attribute);
        match tokens.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => continue,
            Some(other) => panic!("{other:?}"),
            None => break,
        }
    }
    Ok(attributes)
}

fn parse_db(
    span: proc_macro2::Span,
    tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
) -> Result<BackendOption, Error> {
    use BackendOptionVariant::*;
    use ErrorVariant::*;

    match tokens.peek() {
        None => {
            tokens.next();
            return Err(MissingDb.with_span(span));
        }
        Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => {
            tokens.next();
            return Err(MissingDb.with_span(span));
        }
        Some(TokenTree::Punct(punct)) if punct.as_char() == '=' => {
            let punct = punct.span();
            match tokens.nth(1) {
                None => return Err(MissingBackendValue.with_span(punct)),
                Some(TokenTree::Ident(ident)) => {
                    let backend = match ident.to_string().as_str() {
                        "sled" => Sled,
                        "hashmap" => HashMap,
                        "btreemap" => BTreeMap,
                        "trait" => Trait,
                        #[cfg(test)]
                        "test" => Test,
                        _ => return Err(NotABackend(ident).has_span()),
                    };
                    return Ok(BackendOption {
                        backend,
                        span: ident.span(),
                    });
                }
                Some(other) => return Err(InvalidBackendSyntax.with_span(other)),
            }
        }
        Some(other) => return Err(InvalidBackendSyntax.with_span(other)),
    }
}

fn parse_item(tokens: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Result<Options, Error> {
    use ErrorVariant::*;
    let first_token = tokens
        .next()
        .expect("should only get here if peek returned Some");
    match first_token {
        TokenTree::Ident(ident) if ident.to_string() == "db" => {
            let backend = parse_db(ident.span(), tokens)?;
            Ok(Options::Backend(backend))
        }
        TokenTree::Ident(ident) if ident.to_string() == "async" => Ok(Options::Async),
        TokenTree::Ident(ident) => return Err(NotAnOption(ident).has_span()),
        _ => return Err(InvalidSyntax(first_token).has_span()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn parse_db_option_sled() {
        let attr = proc_macro2::TokenStream::from_str("db=sled").unwrap();
        let attribute = parse(attr).unwrap().pop().unwrap();
        assert!(matches!(
            attribute,
            Options::Backend(BackendOption {
                backend: BackendOptionVariant::Sled,
                span: _span
            })
        ));
    }

    #[test]
    fn parse_db_option_btreemap() {
        let attr = proc_macro2::TokenStream::from_str("db=btreemap").unwrap();
        let attribute = parse(attr).unwrap().pop().unwrap();
        assert!(matches!(
            attribute,
            Options::Backend(BackendOption {
                backend: BackendOptionVariant::BTreeMap,
                span: _span
            })
        ));
    }

    #[test]
    fn parse_multiple_option() {
        let attr = proc_macro2::TokenStream::from_str("db=sled,async").unwrap();
        let mut attributes = parse(attr).unwrap();
        assert!(matches!(attributes.pop().unwrap(), Options::Async));
        assert!(matches!(
            attributes.pop().unwrap(),
            Options::Backend(BackendOption {
                backend: BackendOptionVariant::Sled,
                span: _span
            })
        ));
    }
}
