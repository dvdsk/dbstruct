use proc_macro_error::{abort, emit_error, proc_macro_error};
use syn::parse_macro_input;

mod model;
use model::Model;
mod ir;
use ir::Ir;
mod codegen;
use codegen::codegen;
mod errors;
use errors::{GetSpan, Help};

/* TODO: dummy stream? (see proc-macro-error) <dvdsk noreply@davidsk.dev> */

#[proc_macro_attribute]
#[proc_macro_error]
pub fn dbstruct(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as syn::ItemStruct);

    let model = match Model::try_from(input, attr.into()) {
        Ok(model) => model,
        Err(err) => emit_and_abort(err),
    };

    let ir = Ir::from(model);
    let code = codegen(ir);
    code.into()
}

fn emit_and_abort(err: model::Error) -> ! {
    match err {
        model::Error::DbKey(e) => {
            abort!(e.span(), "{}", e)
        }
        model::Error::Field(mut errs) => {
            let last = errs.pop().expect("minimum err vec len is one");
            for e in errs {
                emit_error!(e.span(), e.to_string(); help =? e.help(););
            }
            abort!(last.span(),last.to_string(); help =? last.help(););
        }
        model::Error::Attribute(e) => {
            abort!(e.span(), e.to_string(); help =? e.help(););
        }
        model::Error::Backend(e) => {
            abort!(e.span(), e.to_string(); help =? e.help(););
        }
    }
}

