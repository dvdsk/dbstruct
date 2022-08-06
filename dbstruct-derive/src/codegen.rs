use proc_macro2::TokenStream;
use quote::quote;

use crate::ir::{Accessor, Ir, NewMethod, Struct};

pub fn codegen(ir: Ir) -> TokenStream {
    #![allow(unused_variables)]
    let struct_ident = ir.definition.ident.clone();
    let definition = definition(ir.definition);
    let accessors = accessor_impl(ir.accessors);
    let new_impl = new_impl(ir.new);
    let bounds = ir.bounds;
    quote!(
        #definition

        impl #struct_ident #bounds {
            #new_impl
            #accessors
        }
    )
}

fn new_impl(new: NewMethod) -> TokenStream {
    let NewMethod { fields, vis } = new;

    quote!(
        #vis fn new(ds: DS) -> Result<Self, dbstruct::Error<DS::Error>> {
            Ok(Self {
                ds,
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

    fn test_struct(fields: &[&'static str]) -> Struct {
        let parser = syn::Field::parse_named;
        Struct {
            ident: parse_quote!(Test),
            vis: parse_quote!(pub),
            vars: fields
                .into_iter()
                .map(|s| parser.parse_str(s))
                .map(Result::unwrap)
                .collect(),
        }
    }

    #[test]
    fn output_is_struct_item() {
        let fields = [
            "u8_field: u8",
            "vec_field: Vec<u32>",
            "map_field: HashMap<f32, f64>",
        ];
        let rust = definition(test_struct(&fields));
        println!("{}", rust);
        assert!(syn::parse2::<syn::ItemStruct>(rust).is_ok())
    }

    fn test_accessor() -> Accessor {
        Accessor {
            vis: parse_quote!(pub),
            ident: parse_quote!(queue),
            returns: parse_quote!(dbstruct::wrappers::Vec<u32>),
            body: parse_quote!(dbstruct::wrappers::Vec::new(
                self.ds.clone(),
                2,
                self.queue_len.clone()
            )),
        }
    }

    #[test]
    fn output_is_function_item() {
        let rust = accessor_fn(test_accessor());
        println!("{rust}");
        assert!(syn::parse2::<syn::ItemFn>(rust).is_ok())
    }

    fn test_new_impl() -> NewMethod {
        NewMethod {
            fields: vec![parse_quote!(u8field: 0)],
            vis: parse_quote!(pub),
        }
    }

    #[test]
    fn new_impl_is_function_item() {
        let rust = new_impl(test_new_impl());
        println!("{rust}");
        assert!(syn::parse2::<syn::ItemFn>(rust).is_ok())
    }

    #[test]
    fn code_is_parsable() {
        let ir = Ir {
            definition: test_struct(&["u8field: u8"]),
            new: test_new_impl(),
            accessors: vec![test_accessor()],
            bounds: parse_quote!(where DS: dbstruct::DataStore + std::clone::Clone),
        };
        let rust = codegen(ir);
        println!("{rust}");
        assert!(syn::parse2::<syn::File>(rust).is_ok())
    }

    #[test]
    fn end_to_end() {
        use crate::model::Model;
        use syn::parse_str;

        let input: syn::ItemStruct = parse_str(
            "        
#[dbstruct::dbstruct]
pub struct Test {
    #[dbstruct(Default)]
    the_field: u8,
}",
        )
        .unwrap();

        let model = Model::try_from(input).unwrap();
        let ir = Ir::from(model);
        let rust = codegen(ir);

        println!("{rust}");
        assert!(syn::parse2::<syn::File>(rust).is_ok())
    }
}
