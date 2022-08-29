use proc_macro2::TokenStream;
use quote::quote;

use crate::ir::{Accessor, Ir, NewMethod, Struct};

pub fn codegen(ir: Ir) -> TokenStream {
    #![allow(unused_variables)]
    let struct_ident = ir.definition.ident.clone();
    let definition = definition(ir.definition, &ir.bounds);
    let accessors = accessor_impl(ir.accessors);
    let new_impl = new_impl(ir.new);
    let bounds = ir.bounds;

    match bounds {
        Some(bounds) => quote!(
            #definition

            impl<DS> #struct_ident<DS> #bounds {
                #new_impl
                #accessors
            }
        ),
        None => quote!(
            #definition

            impl #struct_ident {
                #new_impl
                #accessors
            }
        ),
    }
}

fn new_impl(new: NewMethod) -> TokenStream {
    let NewMethod {
        locals,
        fields,
        vis,
        arg,
    } = new;
    quote!(
        #vis fn new(#arg) -> Result<Self, dbstruct::Error<DS::Error>> {
            #(#locals)*
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
        len_vars: extra_vars,
        db,
    } = definition;
    match bounds {
        Some(bounds) => {
            let predicates = &bounds.predicates;
            quote!(
                #vis struct #ident<#predicates> {
                    ds: DS,
                    #(#extra_vars),*
                }
            )
        }
        None => quote!(
        #vis struct #ident {
            #db,
            #(#extra_vars),*
        }),
    }
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
            len_vars: fields
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
            returns: parse_quote!(dbstruct::wrappers::Vec<u32>),
            body: parse_quote!({
                dbstruct::wrappers::Vec::new(self.ds.clone(), 2, self.queue_len.clone())
            }),
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
            locals: Vec::new(),
            arg: parse_quote!(ds: DS),
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
#[dbstruct::dbstruct(db=sled)]
pub struct Test {
    // #[dbstruct(Default)]
    primes: Vec<u32>,
}",
        )
        .unwrap();

        let model = Model::try_from(input).unwrap();
        let ir = Ir::from(model);
        let rust = codegen(ir);

        println!("{rust}");
        assert!(syn::parse2::<syn::File>(rust).is_ok())
    }

    #[test]
    fn fail() {
        use crate::model::Model;
        use syn::parse_str;

        let input: syn::ItemStruct = parse_str(
            r##"        
#[dbstruct::dbstruct(db=sled)]
pub struct Test {
    /// a small list that we dont want structdb to wrap for us
    #[dbstruct(Default)]
    small_list: Vec<u8>,
    /// a small list that we dont want structdb to wrap for us
    #[dbstruct(Default)]
    small_map: HashMap<usize, u32>,
}
"##).unwrap();

        let model = Model::try_from(input).unwrap();
        let ir = Ir::from(model);
        let rust = codegen(ir);

        println!("{rust}");
        assert!(syn::parse2::<syn::File>(rust).is_ok())
    }
}
