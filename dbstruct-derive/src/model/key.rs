use std::collections::HashMap;
use syn::Ident;

type Prefix = u8;
#[derive(Debug)]
pub struct DbKey(HashMap<Ident, Prefix>);

impl DbKey {
    pub(crate) fn new(fields: &syn::Fields) -> Result<Self, &'static str> {
        let mut idents: Vec<_> = fields
            .iter()
            .map(|f| f.ident.clone())
            .map(|i| i.expect("every field must have an ident"))
            .collect();
        idents.sort();
        
        if idents.len() == u8::MAX.into() {
            return Err("A dbstruct can only have 254 fields (implementation limitation)")
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
