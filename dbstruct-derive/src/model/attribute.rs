mod errors;
pub use errors::{Error, ErrorVariant};

pub use super::field::Field;
pub use super::field::Wrapper;
use std::iter::Peekable;

pub use super::key::DbKey;
use proc_macro2::TokenTree;

#[derive(Debug, Clone, Copy)]
pub enum BackendOption {
    Sled,
    Trait,
    #[cfg(test)]
    Test,
}

#[derive(Debug)]
pub enum Options {
    Backend(BackendOption),
    Async,
}

/// attrs is the tokenstream returned by Attribute::parse_args();
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
                        "sled" => BackendOption::Sled,
                        "trait" => BackendOption::Trait,
                        #[cfg(test)]
                        "test" => BackendOption::Test,
                        _ => return Err(NotABackend(ident).has_span()),
                    };
                    return Ok(backend);
                }
                Some(other) => return Err(InvalidBackendFormat.with_span(other)),
            }
        }
        Some(_other) => return Err(InvalidBackendArgs.with_span(_other)),
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
        TokenTree::Ident(ident) => return Err(NotAnAttribute(ident).has_span()),
        _ => return Err(InvalidSyntax(first_token).has_span()),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;

    #[test]
    fn parse_db_option() {
        let attr = proc_macro2::TokenStream::from_str("db=sled").unwrap();
        let attribute = parse(attr).unwrap().pop().unwrap();
        assert!(matches!(attribute, Options::Backend(BackendOption::Sled)));
    }

    #[test]
    fn parse_multiple_option() {
        let attr = proc_macro2::TokenStream::from_str("db=sled,async").unwrap();
        let mut attributes = parse(attr).unwrap();
        assert!(matches!(attributes.pop().unwrap(), Options::Async));
        assert!(matches!(
            attributes.pop().unwrap(),
            Options::Backend(BackendOption::Sled)
        ));
    }
}
