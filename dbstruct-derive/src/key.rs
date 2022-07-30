use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{Ident, Fields};

type Prefix = u8;
pub struct DbKey(HashMap<Ident, Prefix>);

impl DbKey {
    pub(crate) fn new(fields: &Fields) -> Result<Self, &'static str> {
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

    // pub(crate) fn get(&self, ident: &Ident) -> [u8; 1] {
    //     [self.prefix(ident)]
    // }

    fn prefix(&self, ident: &Ident) -> Prefix {
        *self
            .0
            .get(ident)
            .expect("every field's ident should be in the DbKey map")
    }

    pub(crate) fn fn_idx_key(&self, ident: &Ident) -> TokenStream {
        let prefix = self.prefix(ident);

        quote!(
            fn idx_key(idx: u64) -> [u8; 9] {
                let mut res = [0u8; 9];
                res[0] = #prefix;
                res[1..].copy_from_slice(&idx.to_be_bytes());
                res
            }
        )
    }

    pub(crate) fn expr_prefix(&self, ident: &Ident) -> TokenStream {
        let prefix = self.prefix(ident);
        quote!(
            #prefix
        )
    }
}
