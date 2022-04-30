use proc_macro2::{Span, TokenStream};
use proc_macro_error::{abort, abort_call_site, proc_macro_error, set_dummy};
use quote::{format_ident, quote, quote_spanned};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
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

fn basic_methods(field_ident: &Ident, field_type: &Type, key: String) -> TokenStream {
    let getter = field_ident.clone();
    let setter = Ident::new(&format!("set_{}", field_ident), field_ident.span());
    let update = Ident::new(&format!("update_{}", field_ident), field_ident.span());
    let span = field_type.span();

    quote_spanned! {span=>
        #[allow(dead_code)]
        pub fn #setter(&self, position: &#field_type) -> Result<(), structdb::Error> {
            let bytes = bincode::serialize(position)
                .map_err(structdb::Error::Serializing)?;
            self.tree.insert(#key, bytes)?
                .expect("db values should always be set");
            Ok(())
        }

        /// getter for #ident
        /// # Errors
        /// TODO
        #[allow(dead_code)]
        pub fn #getter(&self) -> Result<#field_type, structdb::Error> {
            let bytes = self.tree.get(#key)?
                .expect("db values should always be set");
            Ok(bincode::deserialize(&bytes)
                .map_err(structdb::Error::DeSerializing)?)
        }

        /// # Errors
        /// returns an error incase de or re-serializing failed, in which case the
        /// value of the member in the array will not have changed.
        #[allow(dead_code)]
        pub fn #update(&self, op: impl FnMut(#field_type) -> #field_type + Clone) -> Result<(), structdb::Error> {
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

fn methods_for_field(field: &Field) -> TokenStream {
    let field_type = &field.ty;
    let field_ident = field.ident.as_ref().unwrap();
    let key = field_ident.to_string();

    dbg!(&field_type);
    if !is_option(field_type) {
        let span = field_type.span();
        return quote_spanned!(span=> compile_error!("[structdb] Every type must be contained in an Option"););
    }

    basic_methods(field_ident, field_type, key)
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
    attrs: &[Attribute],
    generics: &Generics,
) -> TokenStream {
    // dbg!(&fields);
    let field_methods: Vec<_> = fields.into_iter().map(methods_for_field).collect();
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
