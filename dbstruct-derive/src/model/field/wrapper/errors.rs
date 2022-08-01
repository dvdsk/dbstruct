use core::fmt;

use proc_macro2::Span;


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

pub trait GetSpan {
    fn span(&self) -> Span;
}

impl GetSpan for proc_macro2::Punct {
    fn span(&self) -> Span {
        proc_macro2::Punct::span(self)
    }
}

impl ErrorVariant {
    pub(super) fn with_span(self, item: impl GetSpan) -> Error {
        Error {
            variant: self,
            span: Some(item.span()),
        }
    }
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

impl Error {
    fn span(&self) -> Span {
        match (self.variant, self.span) {
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
