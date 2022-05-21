use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{Ident, Type};

pub fn methods(
    ident: &proc_macro2::Ident,
    full_type: &Type,
    idx_key_method: TokenStream,
) -> TokenStream {
    let setter = setter(ident, full_type, &idx_key_method);
    let getter = getter(ident, full_type, &idx_key_method);
    let update = update(ident, full_type, &idx_key_method);
    let compare_and_swap = compare_and_swap(ident, full_type, &idx_key_method);

    quote!(
        #setter
        #getter
        #update
        #compare_and_swap
    )
}

fn setter(ident: &proc_macro2::Ident, full_type: &Type, idx_key_method: &TokenStream) -> TokenStream {
    let setter = Ident::new(&format!("set_{}", ident), ident.span());
    let span = ident.span();

    // TODO figure out key, maybe a hash?
    // TODO get previous idx (maybe cache it (starts at 0 anyway))
    // TODO not setter but pusher
    quote_spanned! {span=>
        #[allow(dead_code)]
        pub fn #setter(&self, idx: usize, value: &#full_type) -> std::result::Result<(), dbstruct::Error> {
            #idx_key_method
            let key: [u8; 5] = idx_key(idx);
            let bytes = bincode::serialize(value)
                .map_err(dbstruct::Error::Serializing)?;
            self.tree.insert(key, bytes)?;
            Ok(())
        }
    }
}

pub(crate) fn getter(_ident: &proc_macro2::Ident, _full_type: &Type, _idx_key_method: &TokenStream) -> TokenStream {
    todo!()
}

pub(crate) fn update(_ident: &proc_macro2::Ident, _full_type: &Type, _idx_key_method: &TokenStream) -> TokenStream {
    todo!()
}

pub(crate) fn compare_and_swap(_ident: &proc_macro2::Ident, _full_type: &Type, _idx_key_method: &TokenStream) -> TokenStream {
    todo!()
}

