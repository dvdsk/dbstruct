use proc_macro2::TokenStream;
use proc_macro_error::{abort_call_site, proc_macro_error};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::Data::Struct;
use syn::{Attribute, DataStruct, DeriveInput, Field, Generics, Ident};

use self::key::DbKey;

mod key;
mod methods;

#[proc_macro_attribute]
#[proc_macro_error]
pub fn dbstruct(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(item).unwrap();
    let struct_name = &input.ident;
    let gen = match input.data {
        Struct(DataStruct {
            fields: syn::Fields::Named(ref fields),
            ..
        }) => generate_struct(struct_name, &fields.named, &input.attrs, &input.generics),
        _ => abort_call_site!("dbstruct only supports non-tuple structs"),
    };

    gen.into()
}

fn tree_name(struct_name: &Ident, fields: &Punctuated<Field, Comma>) -> String {
    // use syn::{Type, TypePath};
    let mut res = String::new();
    res.push_str(&struct_name.to_string());
    for field in fields {
        res.push(',');
        let field_ident = field.ident.as_ref().unwrap().to_string();
        res.push_str(&field_ident);
    }
    res
}

fn generate_struct(
    name: &Ident,
    fields: &Punctuated<Field, Comma>,
    _attrs: &[Attribute],
    _generics: &Generics,
) -> TokenStream {
    let keys = DbKey::new(fields);
    let field_methods: Vec<_> = fields
        .into_iter()
        .map(|f| (f, &keys))
        .map(methods::generate)
        .collect();
    let tree_name = tree_name(name, fields);

    quote! {
        struct #name {
            tree: sled::Tree,
        }

        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                write!(f, "dbstruct")
            }
        }

        impl #name {
            pub fn test() -> Result<Self, dbstruct::Error> {
                let db = sled::Config::default()
                    .temporary(true)
                    .open()
                    .unwrap();
                Self::open_tree(db)
            }

            pub fn open_db(path: impl std::convert::AsRef<std::path::Path>) -> Result<Self, dbstruct::Error> {
                let db = sled::Config::default()
                    .path(path)
                    .open()
                    .unwrap();
                Self::open_tree(db)
            }
            pub fn open_tree(db: sled::Db) -> Result<Self, dbstruct::Error> {
                let tree = db.open_tree(#tree_name).unwrap();
                Ok(Self { tree })
            }

            #(#field_methods)*

        }
    }
}
