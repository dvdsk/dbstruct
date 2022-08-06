use proc_macro2::TokenStream;
use quote::quote;

use crate::ir::{Accessor, Ir, NewMethod, Struct};

pub fn codegen(ir: Ir) -> TokenStream {
    #![allow(unused_variables)]
    let struct_ident = ir.definition.ident.clone();
    let r#struct = definition(ir.definition);
    let accessors = accessor_impl(ir.accessors);
    let new_impl = new_impl(ir.new);
    let bounds = ir.bounds;
    quote!(
        r#struct

        impl #struct_ident #bounds {
            #new_impl
            #accessors
        }
    )
}

fn new_impl(new: NewMethod) -> TokenStream {
    let NewMethod { fields, vis } = new;

    quote!(
        #vis fn new(ds: DS) -> Result<Self, dbstruct::Error<DS::Error> {
            Ok(Self {
                #(#fields),*
            })
        }
    )
}

fn accessor_fn(
    Accessor {
        vis,
        ident,
        returns,
        body,
    }: Accessor,
) -> TokenStream {
    quote!(
        #vis fn #ident() -> #returns {
            #body
        }
    )
}

fn accessor_impl(accessors: Vec<Accessor>) -> TokenStream {
    let functions: Vec<_> = accessors.into_iter().map(accessor_fn).collect();
    quote!(
        #(#functions)*
    )
}

fn definition(definition: Struct) -> TokenStream {
    let Struct { ident, vis, vars } = definition;
    quote!(
        #vis struct #ident {
            #(#vars),*
        }
    )
    .into()
}

#[cfg(test)]
mod tests {
    use syn::parse::Parser;
    use syn::parse_quote;

    use super::*;

    #[test]
    fn output_is_struct_item() {
        let parser = syn::Field::parse_named;
        let s = Struct {
            ident: parse_quote!(Test),
            vis: parse_quote!(pub),
            vars: vec![
                parser.parse_str("u8_field: u8").unwrap(),
                parser.parse_str("vec_field: Vec<u32>").unwrap(),
                parser.parse_str("map_field: HashMap<f32, f64>").unwrap(),
            ],
        };

        let rust = definition(s);
        println!("{}", rust);
        assert!(syn::parse2::<syn::ItemStruct>(rust).is_ok())
    }

    #[test]
    fn output_is_function_item() {
        let a = Accessor {
            vis: parse_quote!(pub),
            ident: parse_quote!(queue),
            returns: parse_quote!(dbstruct::wrappers::Vec<u8>),
            body: parse_quote!(dbstruct::wrappers::Vec::new(
                self.ds.clone(),
                2,
                self.queue_len.clone()
            )),
        };

        let rust = accessor_fn(a);
        println!("{rust}");
        assert!(syn::parse2::<syn::ItemFn>(rust).is_ok())
    }
}
