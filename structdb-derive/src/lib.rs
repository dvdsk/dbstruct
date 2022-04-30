use proc_macro2::TokenStream;
use proc_macro_error::{abort_call_site, proc_macro_error};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::Data::Struct;
use syn::{Attribute, DataStruct, DeriveInput, Field, Generics, *};

mod methods;

#[proc_macro_attribute]
#[proc_macro_error]
pub fn structdb(
    _attr: proc_macro::TokenStream,
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

fn impl_structdb(
    name: &Ident,
    fields: &Punctuated<Field, Comma>,
    _attrs: &[Attribute],
    _generics: &Generics,
) -> TokenStream {
    // dbg!(&fields);
    let field_methods: Vec<_> = fields.into_iter().map(methods::generate).collect();
    let tree_name = tree_name(name, fields);

    quote! {
        struct #name {
            tree: sled::Tree,
        }

        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                write!(f, "structdb")
            }
        }

        impl #name {
            pub fn test() -> Result<Self, structdb::Error> {
                let db = sled::Config::default()
                    .temporary(true)
                    .open()
                    .unwrap();
                Self::open_tree(db)
            }

            pub fn open_db(path: impl std::convert::AsRef<std::path::Path>) -> Result<Self, structdb::Error> {
                let db = sled::Config::default()
                    .path(path)
                    .open()
                    .unwrap();
                Self::open_tree(db)
            }
            pub fn open_tree(db: sled::Db) -> Result<Self, structdb::Error> {
                let tree = db.open_tree(#tree_name).unwrap();
                Ok(Self { tree })
            }

            #(#field_methods)*

        }
    }
}
