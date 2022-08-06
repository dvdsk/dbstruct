use proc_macro_error::proc_macro_error;
use syn::parse_macro_input;

mod model;
use model::{Model, DbKey};
mod ir;
use ir::Ir;
mod codegen;
use codegen::codegen;

#[proc_macro_attribute]
#[proc_macro_error]
pub fn dbstruct(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as syn::ItemStruct);
    let keys = DbKey::new(&input.fields).unwrap();
    let model = Model::try_from(input).unwrap();
    let ir = Ir::from(model, &keys);
    let code = codegen(ir);
    code.into()
}
