use core::fmt;
use proc_macro2::Span;

use crate::errors::Help;
use crate::GetSpan;

#[derive(thiserror::Error, Debug)]
pub enum ErrorVariant {
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
    #[error("Option is already initialized as None")]
    OptionNotAllowed,
    #[error("The empty type is not allowed")]
    EmptyTypeForbidden,
    #[error("dbstruct does not know how to represent a missing value for this type")]
    NoDefaultType,
    #[error("Invalid syntax: missing an expression for the default value")]
    MissingDefaultValue,
    #[error("Default value is not an expression")]
    ValueNotExpression(syn::parse::Error),
    #[error("Invalid argument for the Default attribute")]
    InvalidDefaultArg,
    #[error("Types must be fully owned and can not have lifetime params")]
    NotATypeGeneric,
    #[error("{ty} needs {n_needed} generic types")]
    TooFewGenerics{ty: &'static str, n_needed: u8},
    #[error("Too many generics for {ty}, expected {n_needed}")]
    TooManyGenerics{ty: &'static str, n_needed: u8},
    // #[error("Attribute {0} should not have any arguments")]
    // ShouldNotHaveArgs(&'static str),
}

#[derive(thiserror::Error, Debug)]
pub struct Error {
    variant: ErrorVariant,
    span: Option<Span>,
}

impl Help for Error {
    fn help(&self) -> Option<&str> {
        use ErrorVariant::*;
        match self.variant {
            NoDefaultType => Some(
"you can wrap the type in an Option, add an attribute to use the Default trait: 
`#[dbstruct(Default)]` or provide an expression to generate a default 
value: `#[dbstruct(Default=<expr>)]"),
            InvalidSyntax(_) => Some("try one of these: `#[dbstruct(Default)]`, `#[dbstruct(Default=\"<expr>\"]`"),
            OptionNotAllowed => Some("try removing the attribute"),
            MultipleWrapperAttributes => Some("when using Default=\"<expr>\" make sure the <expr> string is properly escaped"),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.variant.fmt(f)
    }
}

impl Error {
    pub fn span(&self) -> Span {
        match (&self.variant, self.span) {
            (ErrorVariant::NotAWrapper(item), None) => item.span(),
            (ErrorVariant::InvalidSyntax(item), None) => item.span(),
            (ErrorVariant::ValueNotExpression(item), None) => item.span(),
            (_, Some(span)) => span,
            (_var, _) => panic!(
                "error should track a span for {_var:?} as 
                the variant itself does not contain one"
            ),
        }
    }
}

impl GetSpan for super::Attribute {
    fn span(&self) -> Span {
        use super::Attribute::*;
        match self {
            // NoWrap { span } => *span,
            DefaultTrait { span } => *span,
            DefaultValue { expr } => syn::spanned::Spanned::span(expr),
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
