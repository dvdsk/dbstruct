use proc_macro2::TokenStream;
use quote::quote;

use crate::ir::{Ir, Struct};

pub fn codegen(ir: Ir) -> TokenStream {
    let r#struct = definition(ir.definition);
    quote!(r#struct)
}

fn definition(definition: Struct) -> TokenStream {
    let Struct { ident, vis, vars } = definition;
    quote!(
        #vis struct #ident {
            #(#vars),*
        }
    ).into()
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
        dbg!(&rust);
        assert!(syn::parse2::<syn::ItemStruct>(rust).is_ok())
        
    }
}
