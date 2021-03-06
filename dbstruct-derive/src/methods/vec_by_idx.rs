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
                        let curr = u64::from_be_bytes(array);
                        curr + 1
                    }
                    None => 0,
                };
                Some(number.to_be_bytes().to_vec())
            }

            let last_idx = self.tree.get_lt([prefix+1])?
                .map(|(key, _)| u64::from_be_bytes(
                        key[1..]
                            .try_into()
                            .expect("vector keys need to be prefix + valid u64 as be bytes")
                        )
                    )
                .unwrap_or(0);
            let key: [u8; 9] = idx_key(last_idx + 1);
            let bytes = bincode::serialize(value).map_err(dbstruct::Error::Serializing)?;
            self.tree.insert(key, bytes)?;
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
            let last_element = match self.tree.get_lt([prefix+1])?.map(|(key, _)| key) {
                Some(key) => key,
                None => return Ok(None),
            };

            let bytes = match self.tree.remove(last_element)? {
                Some(bytes) => bytes,
                None => return Ok(None), // value must been deleted between fetching len and this
            };
            let value = bincode::deserialize(&bytes).map_err(dbstruct::Error::DeSerializing)?;
            Ok(Some(value))
        }
    }
}
