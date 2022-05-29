use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{Ident, Type};

pub fn methods(
    ident: &proc_macro2::Ident,
    full_type: &Type,
    fn_key_method: TokenStream,
    expr_prefix: TokenStream,
) -> TokenStream {
    let setter = push(ident, full_type, &fn_key_method, &expr_prefix);
    let getter = pop(ident, full_type, &fn_key_method, &expr_prefix);

    quote!(
        #setter
        #getter
    )
}

fn push(
    ident: &proc_macro2::Ident,
    full_type: &Type,
    fn_key_method: &TokenStream,
    expr_prefix: &TokenStream,
) -> TokenStream {
    let fn_name = Ident::new(&format!("push_{}", ident), ident.span());
    let span = ident.span();

    quote_spanned! {span=>
        /// atomically push a new value into the db.
        #[allow(dead_code)]
        pub fn #fn_name(&self, value: &#full_type) -> std::result::Result<(), dbstruct::Error> {
            #fn_key_method
            let prefix = #expr_prefix;

            fn increment(old: Option<&[u8]>) -> Option<Vec<u8>> {
                let number = match old {
                    Some(bytes) => {
                        let err = "db should contain 8 long byte slice representing the vec idx at key #prefix";
                        let array: [u8; 8] = bytes.try_into().expect(err);
                        let curr = u64::from_be_bytes(bytes);
                        curr + 1
                    }
                    None => 0,
                };
                Some(number.to_be_bytes().to_vec())
            }

            let idx = self.tree.fetch_and_update([prefix], increment)?;
            let key: [u8; 9] = idx_key(idx);
            let bytes = bincode::serialize(value).map_err(dbstruct::Error::Serializing)?;
            let res = self.tree.insert(key, bytes)?;
            Ok(())
        }
    }
}

pub(crate) fn pop(
    ident: &proc_macro2::Ident,
    full_type: &Type,
    fn_key_method: &TokenStream,
    expr_prefix: &TokenStream,
) -> TokenStream {
    let fn_name = Ident::new(&format!("pop_{}", ident), ident.span());
    let span = ident.span();

    quote_spanned! {span=>
        /// atomically pop a new value from the db
        #[allow(dead_code)]
        pub fn #fn_name(&self) -> std::result::Result<Option<#full_type>, dbstruct::Error> {
            #fn_key_method
            let prefix = #expr_prefix;
            let bytes = match self.get([prefix])? {
                Some(bytes) => bytes,
                None => return None, // no len is zero len
            };

            let err = "db should contain 8 long byte slice representing the vec idx at key #prefix";
            let array: [u8; 8] = bytes.try_into().expect(err);
            let idx = u64::from_be_bytes(bytes);

            let key: [u8; 9] = idx_key(len - 1);
            let bytes = match self.tree.remove(key)? {
                Some(bytes) => bytes,
                None => return None, // value must been deleted between fetching len and this
            };
            let value = bincode::deserialize(bytes).map_err(dbstruct::Error::DeSerializing)?;
            Ok(Some(value))
        }
    }
}
