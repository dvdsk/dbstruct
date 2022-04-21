use proc_macro2::{Span, TokenStream};
use proc_macro_error::{abort, abort_call_site, proc_macro_error, set_dummy};
use quote::{format_ident, quote, quote_spanned};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::Data::Struct;
use syn::{Attribute, DataStruct, DeriveInput, Field, Generics, *};

#[proc_macro_attribute]
#[proc_macro_error]
pub fn structdb(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(item).unwrap();
    let struct_name = &input.ident;
    let gen = match input.data {
        Struct(DataStruct {
            fields: syn::Fields::Named(ref fields),
            ..
        }) => impl_structdb(struct_name, &fields.named, &input.attrs, &input.generics),
        _ => abort_call_site!("structdb only supports non-tuple structs"),
    };

    gen.into()
}

fn impl_structdb(
    name: &Ident,
    fields: &Punctuated<Field, Comma>,
    attrs: &[Attribute],
    generics: &Generics,
) -> TokenStream {

    dbg!(&fields);
    quote! {
        struct #name {
            db: sled::Db,
        }

        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                write!(f, "structdb")
            }
        }

        impl #name {
            pub fn new() -> Self {
                let db = sled::Config::default().temporary(true).open().unwrap();
                Self { db }
            }

            pub fn set_position(&self, position: u32) {
                dbg!("it works!");
            }
        }
    }
}
