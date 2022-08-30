use core::fmt;
use proc_macro2::{Span, TokenTree};

use crate::errors::Help;
use crate::GetSpan;

#[derive(Debug, thiserror::Error)]
pub enum ErrorVariant {
    #[error("no database backend specified")]
    MissingDb,
    #[error("backend option has no value set")]
    MissingBackendValue,
    #[error("incorrect syntax for backend")]
    InvalidBackendSyntax,
    #[error("not a known dbstruct option")]
    NotAnOption(proc_macro2::Ident),
    #[error("invalid syntax for dbstruct option")]
    InvalidSyntax(TokenTree),
    #[error("Not a known database backend: `{0}`")]
    NotABackend(proc_macro2::Ident),
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
    fn help(&self) -> Option<String> {
        use ErrorVariant::*;
        Some(match self.variant {
            MissingDb => "try specifying an db, for example: `db=sled`",
            MissingBackendValue => "try setting a supported backend, for example `db=sled`",
            InvalidBackendSyntax => "a backend should be a single world not enclosed in \"",
            NotAnOption(_) => "the only supported option currently is: db",
            InvalidSyntax(_) => "the option should be a single word not enclosed in \"",
            NotABackend(_) => "try sled as database backend",
        }.to_owned())
    }
}

impl Error {
    pub fn span(&self) -> Span {
        use ErrorVariant::*;
        match (&self.variant, self.span) {
            (NotAnOption(item), None) => item.span(),
            (InvalidSyntax(item), None) => item.span(),
            (NotABackend(item), None) => item.span(),
            (_, Some(span)) => span,
            (_var, _) => unreachable!(
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
