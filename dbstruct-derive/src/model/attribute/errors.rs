use core::fmt;
use proc_macro2::{Span, TokenTree};

use crate::errors::Help;
use crate::GetSpan;

#[derive(Debug, thiserror::Error)]
pub enum ErrorVariant {
    #[error("You must choose which database to wrap")]
    NoBackendSpecified,
    #[error("todo")]
    MissingDb,
    #[error("todo")]
    MissingBackendValue,
    #[error("todo")]
    InvalidBackendFormat,
    #[error("todo")]
    InvalidBackendArgs,
    #[error("todo")]
    NotAnAttribute(proc_macro2::Ident),
    #[error("todo")]
    InvalidSyntax(TokenTree),
    #[error("Not a known dbstruct backend: `{0}`")]
    NotABackend(proc_macro2::Ident),
    #[error("Invalid token tree expected Group")]
    InvalidTokenTree,
}

#[derive(thiserror::Error, Debug)]
pub struct Error {
    variant: ErrorVariant,
    span: Option<Span>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.variant.fmt(f)
    }
}

impl Help for Error {
    fn help(&self) -> Option<&str> {
        todo!()
    }
}

impl Error {
    pub fn span(&self) -> Span {
        match (&self.variant, self.span) {
            (ErrorVariant::NotAnAttribute(item), None) => item.span(),
            (ErrorVariant::InvalidSyntax(item), None) => item.span(),
            (_, Some(span)) => span,
            (_var, _) => panic!(
                "error should track a span for {_var:?} as 
                the variant itself does not contain one"
            ),
        }
    }
}

impl ErrorVariant {
    pub(super) fn with_span(self, item: impl GetSpan) -> Error {
        Error {
            variant: self,
            span: Some(item.span()),
        }
    }

    pub(super) fn has_span(self) -> Error {
        Error {
            variant: self,
            span: None,
        }
    }
}
