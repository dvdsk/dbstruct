use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::{Ident, Type};

pub(super) fn setter(field_ident: &Ident, field_type: &Type, key: &str) -> TokenStream {
    let setter = Ident::new(&format!("set_{}", field_ident), field_ident.span());
    let span = field_type.span();

    quote_spanned! {span=>
        #[allow(dead_code)]
        pub fn #setter(&self, position: &#field_type) -> std::result::Result<(), structdb::Error> {
            let bytes = bincode::serialize(position)
                .map_err(structdb::Error::Serializing)?;
            self.tree.insert(#key, bytes)?
                .expect("db values should always be set");
            Ok(())
        }
    }
}

pub(super) fn getter(field_ident: &Ident, field_type: &Type, key: &str) -> TokenStream {
    let getter = field_ident.clone();
    let span = field_type.span();

    quote_spanned! {span=>
        /// getter for #ident
        /// # Errors
        /// TODO
        #[allow(dead_code)]
        pub fn #getter(&self) -> std::result::Result<#field_type, structdb::Error> {
            let bytes = self.tree.get(#key)?
                .expect("db values should always be set");
            Ok(bincode::deserialize(&bytes)
                .map_err(structdb::Error::DeSerializing)?)
        }
    }
}

pub(super) fn update(field_ident: &Ident, field_type: &Type, key: &str) -> TokenStream {
    let update = Ident::new(&format!("update_{}", field_ident), field_ident.span());
    let span = field_type.span();

    quote_spanned! {span=>
        /// # Errors
        /// returns an error incase de or re-serializing failed, in which case the
        /// value of the member in the array will not have changed.
        #[allow(dead_code)]
        pub fn #update(&self, op: impl FnMut(#field_type) -> #field_type + Clone)
            -> std::result::Result<(), structdb::Error> {

            let mut res = Ok(());
            let update = |old: Option<&[u8]>| {
                let old = old.expect("db values should always be set");
                match bincode::deserialize(old) {
                    Err(e) => {
                        res = Err(structdb::Error::DeSerializing(e));
                        Some(old.to_vec())
                    }
                    Ok(v) => {
                        let new = op.clone()(v);
                        match bincode::serialize(&new) {
                            Ok(new_bytes) => Some(new_bytes),
                            Err(e) => {
                                res = Err(structdb::Error::Serializing(e));
                                Some(old.to_vec())
                            }
                        }
                    }
                }
            };
            let _bytes = self.tree.update_and_fetch(#key, update)?
                .expect("db values should always be set");
            // res
            Ok(())
        }
    }
}

pub(super) fn compare_and_swap(field_ident: &Ident, field_type: &Type, key: &str) -> TokenStream {
    let compare_and_swap = Ident::new(
        &format!("compare_and_swap_{}", field_ident),
        field_ident.span(),
    );
    let span = field_type.span();

    quote_spanned! {span=>
        #[allow(dead_code)]
        pub fn #compare_and_swap(&self, old: #field_type, new: #field_type)
            -> std::result::Result<
                std::result::Result<(), structdb::CompareAndSwapError<#field_type>>,
            structdb::Error> {
            let old = bincode::serialize(&old).map_err(structdb::Error::Serializing)?;
            let new = bincode::serialize(&new).map_err(structdb::Error::Serializing)?;
            Ok(match self.tree.compare_and_swap(#key, Some(old), Some(new))? {
                Ok(()) => Ok(()),
                Err(e) => Err(e.try_into()?),
            })
        }
    }
}
