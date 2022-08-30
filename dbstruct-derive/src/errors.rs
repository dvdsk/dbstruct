use proc_macro2::Span;

pub trait Help {
    fn help(&self) -> Option<String>;
}

pub trait GetSpan {
    fn span(&self) -> Span;
}

impl GetSpan for proc_macro2::Span {
    fn span(&self) -> Span {
        *self
    }
}

macro_rules! impl_getspan {
    ($type:ty) => {
        impl GetSpan for $type {
            fn span(&self) -> Span {
                self.span()
            }
        }
        impl GetSpan for &$type {
            fn span(&self) -> Span {
                (*self).span()
            }
        }
    };
}

impl_getspan!(proc_macro2::Punct);
impl_getspan!(proc_macro2::Literal);
impl_getspan!(proc_macro2::TokenTree);
impl_getspan!(proc_macro2::Ident);

macro_rules! impl_getspan_syn {
    ($type:ty) => {
        impl GetSpan for $type {
            fn span(&self) -> Span {
                syn::spanned::Spanned::span(self)
            }
        }
        impl GetSpan for &$type {
            fn span(&self) -> Span {
                syn::spanned::Spanned::span(*self)
            }
        }
    }
}

impl_getspan_syn!(syn::Type);
impl_getspan_syn!(syn::Attribute);
