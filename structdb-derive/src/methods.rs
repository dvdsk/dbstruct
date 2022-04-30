use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{Field, *};

pub mod basic;

fn is_option(type_path: &Type) -> bool {
    let outer_type = match type_path {
        Type::Path(TypePath { path, .. }) => path
            .segments
            .iter()
            .next()
            .expect("Type path should be at least one segment"),
        _ => unreachable!(),
    };

    outer_type.ident == "Option"
}

pub fn generate(field: &Field) -> TokenStream {
    let field_type = &field.ty;
    let field_ident = field.ident.as_ref().unwrap();
    let key = &field_ident.to_string();

    if !is_option(field_type) {
        let span = field_type.span();
        return quote_spanned!(span=> compile_error!("[structdb] Every type must be contained in an Option"););
    }

    let setter = basic::setter(field_ident, field_type, key);
    let getter = basic::getter(field_ident, field_type, key);
    let update = basic::update(field_ident, field_type, key);
    let compare_and_swap = basic::compare_and_swap(field_ident, field_type, key);

    quote!(
        #setter
        #getter
        #update
        #compare_and_swap
    )
}
