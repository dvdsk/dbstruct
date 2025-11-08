use core::fmt;
use std::collections::HashSet;

use proc_macro2::Span;

use crate::errors::{GetSpan, Help};
use crate::model::attribute::BackendOptionVariant;

use super::attribute::Options;
use super::{attribute, Field};

#[derive(Debug, thiserror::Error)]
pub enum ErrorVariant {
    #[error("multiple database backends specified")]
    MultipleBackends,
    #[error("No database backend specified to use as backend")]
    NoBackendSpecified,
    #[error(
        "The database backend ({backend}) you specified can not support all the structs fields"
    )]
    MissesTraits {
        backend: Backend,
        needed: HashSet<ExtraBound>,
    },
}

impl ErrorVariant {
    pub(super) fn with_span(self, item: impl GetSpan) -> Error {
        Error {
            variant: self,
            span: item.span(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub struct Error {
    variant: ErrorVariant,
    span: Span,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.variant.fmt(f)
    }
}

impl Help for Error {
    fn help(&self) -> Option<String> {
        use ErrorVariant::*;
        Some(match &self.variant {
            MultipleBackends => "remove one of the backends".to_owned(),
            NoBackendSpecified => "specify a backend using #[dbstruct(db=sled)]".to_owned(),
            MissesTraits { needed, .. } => {
                let compatible = Backend::provided()
                    .into_iter()
                    .filter(|b| b.traits().is_superset(needed))
                    .map(|b| b.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("You need a backend that implements all of these traits: {needed:?}.\nDatabase backends that implement those traits: {compatible}")
            }
        })
    }
}

impl Error {
    pub fn span(&self) -> Span {
        self.span
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum ExtraBound {
    Atomic,
    Ordered,
}

#[derive(Debug, Clone)]
pub enum Backend {
    Sled,
    HashMap,
    BTreeMap,
    Trait {
        bounds: Vec<ExtraBound>,
    },
    #[cfg(test)]
    Test,
}

impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Backend::Sled => write!(f, "sled"),
            Backend::HashMap => write!(f, "hashmap"),
            Backend::BTreeMap => write!(f, "btreemap"),
            Backend::Trait { .. } => write!(f, "trait"),
            #[cfg(test)]
            Backend::Test => unreachable!(),
        }
    }
}

impl Backend {
    pub fn try_from(options: &[attribute::Options], fields: &[Field]) -> Result<Backend, Error> {
        use BackendOptionVariant::*;
        use ErrorVariant::*;

        let mut backends = options.iter().filter_map(|opt| match opt {
            Options::Backend(b) => Some(b),
            _ => None,
        });

        let backend_option = match (backends.next(), backends.next()) {
            (None, _) => return Err(NoBackendSpecified.with_span(Span::call_site())),
            (Some(b), None) => *b,
            (Some(b0), Some(b1)) => {
                let span = b0.span.join(b1.span).unwrap_or(b1.span);
                return Err(MultipleBackends.with_span(span));
            }
        };

        let backend = match backend_option.backend {
            Trait => {
                return Ok(Backend::Trait {
                    bounds: fields
                        .iter()
                        .flat_map(|f| f.wrapper.needed_traits().into_iter())
                        .collect(),
                })
            }
            HashMap => Backend::HashMap,
            BTreeMap => Backend::BTreeMap,
            Sled => Backend::Sled,
            #[cfg(test)]
            Test => Backend::Test,
        };

        for field in fields {
            let needed = field.wrapper.needed_traits();
            let missing: HashSet<_> = needed.difference(&backend.traits()).copied().collect();
            if !missing.is_empty() {
                return Err(MissesTraits { backend, needed }.with_span(backend_option.span));
            }
        }

        Ok(backend)
    }

    fn traits(&self) -> HashSet<ExtraBound> {
        use ExtraBound::*;
        match self {
            Backend::Sled => vec![Atomic, Ordered].into_iter(),
            Backend::HashMap => vec![].into_iter(),
            Backend::BTreeMap => vec![Atomic, Ordered].into_iter(),
            Backend::Trait { .. } => unreachable!("should never be called when backend is Trait"),
            #[cfg(test)]
            Backend::Test => vec![].into_iter(),
        }
        .collect()
    }

    fn provided() -> [Backend; 2] {
        [Backend::Sled, Backend::HashMap]
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;
    use ExtraBound::*;

    use crate::model::attribute::{BackendOption, BackendOptionVariant};
    use crate::model::Wrapper;

    use super::*;

    #[test]
    fn err_on_unsupported_backend() {
        let options = [Options::Backend(BackendOption {
            backend: BackendOptionVariant::Test,
            span: Span::call_site(),
        })];
        let fields = [Field {
            ident: parse_quote!(test_a),
            vis: parse_quote!(pub),
            wrapper: Wrapper::Vec {
                ty: parse_quote!(u8),
            },
            key: 1,
        }];
        let err = Backend::try_from(&options, &fields).unwrap_err();
        match err.variant {
            ErrorVariant::MissesTraits {
                needed: missing, ..
            } => {
                let correct = [Ordered].into_iter().collect();
                assert_eq!(missing, correct);
            }
            _ => unreachable!("expected error missingtraits got: {err:?}"),
        }
    }

    #[test]
    fn supported_backend() {
        let options = [Options::Backend(BackendOption {
            backend: BackendOptionVariant::Sled,
            span: Span::call_site(),
        })];

        let fields = [Field {
            ident: parse_quote!(test_a),
            vis: parse_quote!(pub),
            wrapper: Wrapper::Vec {
                ty: parse_quote!(u8),
            },
            key: 1,
        }];
        let backend = Backend::try_from(&options, &fields).unwrap();
        assert!(matches!(backend, Backend::Sled));
    }

    #[test]
    fn reject_double_backend() {
        let span = Span::call_site();
        let options = [
            Options::Backend(BackendOption {
                backend: BackendOptionVariant::Sled,
                span,
            }),
            Options::Backend(BackendOption {
                backend: BackendOptionVariant::Sled,
                span,
            }),
        ];
        let fields = [Field {
            ident: parse_quote!(test_a),
            vis: parse_quote!(pub),
            wrapper: Wrapper::Vec {
                ty: parse_quote!(u8),
            },
            key: 1,
        }];
        let err = Backend::try_from(&options, &fields).unwrap_err();
        assert!(matches!(err.variant, ErrorVariant::MultipleBackends));
    }
}
