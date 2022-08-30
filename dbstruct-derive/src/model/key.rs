use std::collections::HashMap;
use syn::Ident;

use crate::errors::GetSpan;

#[derive(thiserror::Error, Debug)]
#[error("A dbstruct can only have 254 fields")]
pub struct Error {
    span: proc_macro2::Span,
}

impl GetSpan for Error {
    fn span(&self) -> proc_macro2::Span {
        self.span
    }
}

type Prefix = u8;
#[derive(Debug)]
pub struct DbKey(HashMap<Ident, Prefix>);

impl DbKey {
    pub(crate) fn new(fields: &syn::Fields) -> Result<Self, Error> {
        let mut idents: Vec<_> = fields
            .iter()
            .map(|f| f.ident.clone())
            .map(|i| i.expect("should already be verified this is a named struct"))
            .collect();
        idents.sort();

        if let Some(ident) = idents.get(u8::MAX as usize) {
            return Err(Error { span: ident.span() });
        };

        let map = idents
            .into_iter()
            .enumerate()
            .map(|(id, ident)| (ident, id as Prefix))
            .collect();
        Ok(Self(map))
    }

    pub fn prefix(&self, ident: &Ident) -> Prefix {
        *self
            .0
            .get(ident)
            .expect("every field's ident should be in the DbKey map")
    }
}
