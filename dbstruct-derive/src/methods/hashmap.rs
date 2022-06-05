use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{Ident, Type};

pub fn methods(
    ident: &proc_macro2::Ident,
    key_type: &Type,
    value_type: &Type,
    expr_prefix: TokenStream,
) -> TokenStream {
    let setter = get(ident, key_type, value_type, &expr_prefix);
    let getter = insert(ident, key_type, value_type, &expr_prefix);

    quote!(
        #setter
        #getter
    )
}


fn get(
    ident: &proc_macro2::Ident,
    key_type: &Type,
    value_type: &Type,
    expr_prefix: &TokenStream,
) -> TokenStream {
    let fn_name = Ident::new(&format!("set_{}", ident), ident.span());
    let span = ident.span();

    quote_spanned! {span=>
        /// atomically push a new value into the db.
        #[allow(dead_code)]
        pub fn #fn_name(&self, key: &#key_type, value: &#value_type) 
            -> std::result::Result<Option<#key_type>, dbstruct::Error> 
        {
            let prefix = #expr_prefix;
            let mut key_buffer = Vec::new();
            key_buffer.push(prefix);
            let mut key_buffer = std::io::Cursor::new(key_buffer);
            bincode::serialize_into(&mut key_buffer, key).map_err(dbstruct::Error::Serializing)?;
            let key_bytes = key_buffer.into_inner();
            let value_bytes = bincode::serialize(value).map_err(dbstruct::Error::Serializing)?;

            let existing = match self.tree.insert(key_bytes, value_bytes)? {
                Some(bytes) => bincode::deserialize(&bytes).map_err(dbstruct::Error::DeSerializing)?,
                None => None,
            };
            Ok(existing)
        }
    }
}

pub(crate) fn insert(
    ident: &proc_macro2::Ident,
    key_type: &Type,
    value_type: &Type,
    expr_prefix: &TokenStream,
) -> TokenStream {
    let fn_name = Ident::new(&format!("get_{}", ident), ident.span());
    let span = ident.span();

    quote_spanned! {span=>
        /// atomically pop a new value from the db
        #[allow(dead_code)]
        pub fn #fn_name(&self, key: &#key_type) 
            -> std::result::Result<Option<#value_type>, dbstruct::Error> 
        {
            let prefix = #expr_prefix;
            let mut key_buffer = Vec::new();
            key_buffer.push(prefix);
            let mut key_buffer = std::io::Cursor::new(key_buffer);
            bincode::serialize_into(&mut key_buffer, key).map_err(dbstruct::Error::Serializing)?;
            let key_bytes = key_buffer.into_inner();

            let value = match self.tree.get(key_bytes)? {
                Some(bytes) => bincode::deserialize(&bytes).map_err(dbstruct::Error::DeSerializing)?,
                None => None,
            };
            Ok(value)
        }
    }
}
