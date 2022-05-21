use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Field, Ident};

type Prefix = u8;
pub struct DbKey(HashMap<Ident, Prefix>);

impl DbKey {
    pub(crate) fn new(fields: &Punctuated<Field, Comma>) -> Self {
        let mut idents: Vec<_> = fields
            .iter()
            .map(|f| f.ident.clone())
            .map(|i| i.expect("every field must have an ident"))
            .collect();
        idents.sort();
        let map = idents
            .into_iter()
            .enumerate()
            .map(|(id, ident)| (ident, id as Prefix))
            .collect();
        Self(map)
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

    pub(crate) fn idx_key_method(&self, ident: &Ident) -> TokenStream {
        let prefix = self.prefix(ident);

        quote!(
            fn idx_key(idx: usize) -> [u8; 5] {
                let mut res = [#prefix, 0u8,0u8,0u8,0u8];
                res[1..].copy_from_slice(&idx.to_be_bytes());
                res
            }
        )
    }
}
