use proc_macro2::TokenStream;
use quote::quote;

use crate::ir::{Accessor, Ir, NewMethod, Struct};

pub fn codegen(ir: Ir) -> TokenStream {
    #![allow(unused_variables)]
    let struct_ident = ir.definition.ident.clone();
    let definition = definition(ir.definition, &ir.bounds);
    let accessors = accessor_impl(ir.accessors);
    let new_impls = new_impls(ir.new);
    let bounds = ir.bounds;

    match bounds {
        Some(bounds) => quote!(
            #definition

            impl<DS> #struct_ident<DS> #bounds {
                #new_impls
                #accessors
            }
        ),
        None => quote!(
            #definition

            impl #struct_ident {
                #new_impls
                #accessors
            }
        ),
    }
}

fn new_impls(new: impl IntoIterator<Item = NewMethod>) -> TokenStream {
    new.into_iter()
        .map(
            |NewMethod {
                 locals,
                 members,
                 vis,
                 arg,
                 error_ty,
                 name,
             }| {
                quote!(
                     #vis fn #name(#arg) -> Result<Self, ::dbstruct::Error<#error_ty>> {
                         #(#locals)*
                         Ok(Self {
                             ds,
                             #(#members),*
                         })
                     }
                )
            },
        )
        .collect()
}

fn accessor_fn(
    Accessor {
        vis,
        ident,
        returns,
        body,
    }: Accessor,
) -> TokenStream {
    quote!(#vis fn #ident(&self) -> #returns #body)
}

fn accessor_impl(accessors: Vec<Accessor>) -> TokenStream {
    let functions: Vec<_> = accessors.into_iter().map(accessor_fn).collect();
    quote!(
        #(#functions)*
    )
}

fn definition(definition: Struct, bounds: &Option<syn::WhereClause>) -> TokenStream {
    let Struct {
        ident,
        vis,
        member_vars,
        db,
    } = definition;
    match bounds {
        Some(bounds) => {
            let predicates = &bounds.predicates;
            quote!(
                #vis struct #ident<#predicates> {
                    ds: DS,
                    #(#member_vars),*
                }
            )
        }
        None => quote!(
        #vis struct #ident {
            #db,
            #(#member_vars),*
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use itertools::Itertools;
    use syn::parse::Parser;
    use syn::parse_quote;

    use super::*;

    fn test_struct(fields: &[&'static str]) -> Struct {
        let parser = syn::Field::parse_named;
        Struct {
            ident: parse_quote!(Test),
            vis: parse_quote!(pub),
            member_vars: fields
                .into_iter()
                .map(|s| parser.parse_str(s))
                .map(Result::unwrap)
                .collect(),
            db: parser.parse_str("ds: DS").unwrap(),
        }
    }

    fn test_bounds() -> syn::WhereClause {
        parse_quote!(where DS: dbstruct::DataStore + Clone)
    }

    #[test]
    fn output_is_struct_item() {
        let fields = [
            "u8_field: u8",
            "vec_field: Vec<u32>",
            "map_field: HashMap<f32, f64>",
        ];
        let rust = definition(test_struct(&fields), &Some(test_bounds()));
        println!("{}", rust);
        assert!(syn::parse2::<syn::ItemStruct>(rust).is_ok())
    }

    fn test_accessor() -> Accessor {
        Accessor {
            vis: parse_quote!(pub),
            ident: parse_quote!(queue),
            returns: parse_quote!(dbstruct::wrapper::Vec<u32>),
            body: parse_quote!({
                dbstruct::wrapper::Vec::new(self.ds.clone(), 2, self.queue_len.clone())
            }),
        }
    }

    #[test]
    fn output_is_function_item() {
        let rust = accessor_fn(test_accessor());
        println!("{rust}");
        assert!(syn::parse2::<syn::ItemFn>(rust).is_ok())
    }

    fn test_new_impls() -> Vec<NewMethod> {
        ["new", "secondary_new"]
            .into_iter()
            .map(|name| NewMethod {
                members: vec![parse_quote!(u8field: 0)],
                vis: parse_quote!(pub),
                locals: Vec::new(),
                arg: Some(parse_quote!(ds: DS)),
                error_ty: parse_quote!(DS),
                name: syn::parse_str(name).unwrap(),
            })
            .collect()
    }

    #[test]
    fn new_impls_are_function_item() {
        let token_tree = new_impls(test_new_impls());
        let token_tree = token_tree.into_iter().collect_vec();
        for new_impl in token_tree
            .split(|token| token.to_string().as_str() == "pub")
            .filter(|tokens| tokens.len() > 1)
        {
            let stream = TokenStream::from_iter(new_impl.into_iter().cloned());
            assert!(syn::parse2::<syn::ItemFn>(stream).is_ok())
        }
    }

    #[test]
    fn code_is_parsable() {
        let ir = Ir {
            definition: test_struct(&["u8field: u8"]),
            new: test_new_impls(),
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

        let input_attr = proc_macro2::TokenStream::from_str("db=sled").unwrap();
        let input_struct: syn::ItemStruct = parse_str(
            "
pub struct Test {
    // #[dbstruct(Default)]
    primes: Vec<u32>,
}",
        )
        .unwrap();

        let model = Model::try_from(input_struct, input_attr).unwrap();
        let ir = Ir::from(model);
        let rust = codegen(ir);

        println!("{rust}");
        assert!(syn::parse2::<syn::File>(rust).is_ok())
    }

    #[test]
    fn default_vec_and_map() {
        use crate::model::Model;
        use syn::parse_str;

        let input_attr = proc_macro2::TokenStream::from_str("db=sled").unwrap();
        let input_struct: syn::ItemStruct = parse_str(
            r##"
pub struct Test {
    /// a small list that we dont want structdb to wrap for us
    #[dbstruct(Default)]
    small_list: Vec<u8>,
    #[dbstruct(Default)]
    small_map: HashMap<usize, u32>,
}
"##,
        )
        .unwrap();

        let model = Model::try_from(input_struct, input_attr).unwrap();
        let ir = Ir::from(model);
        let rust = codegen(ir);

        println!("{rust}");
        assert!(syn::parse2::<syn::File>(rust).is_ok())
    }
}
