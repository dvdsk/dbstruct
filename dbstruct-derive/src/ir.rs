mod accessor;
mod new_method;
mod struct_def;

pub use accessor::Accessor;
pub use new_method::NewMethod;
pub use struct_def::Struct;
use syn::parse_quote;

use crate::model::backend::{Backend, ExtraBound};
use crate::model::Model;

pub struct Ir {
    pub definition: Struct,
    pub new: NewMethod,
    pub accessors: Vec<Accessor>,
    pub bounds: Option<syn::WhereClause>,
}

fn bound_to_ir(bound: &ExtraBound) -> syn::TraitBound {
    match bound {
        ExtraBound::Atomic => parse_quote!(dbstruct::traits::data_store::Atomic),
        ExtraBound::Orderd => parse_quote!(dbstruct::traits::data_store::Orderd),
    }
}

fn bounds_from(model: &Model) -> Option<syn::WhereClause> {
    match &model.backend {
        Backend::Trait { bounds } => {
            let bounds = bounds.iter().map(bound_to_ir);
            parse_quote!(where DS: dbstruct::DataStore + std::clone::Clone + #(#bounds),*)
        }
        _ => None,
    }
}

fn backend_type(backend: &Backend) -> syn::Type {
    match backend {
        Backend::Sled => parse_quote!(::dbstruct::sled::Tree),
        Backend::Trait { .. } => parse_quote!(DS),
        #[cfg(test)]
        Backend::Test => unreachable!("Test backend is not supported for codegen"),
    }
}

impl Ir {
    pub fn from(model: Model) -> Self {
        let definition = Struct::from(&model);
        let new = NewMethod::from(&model, &definition);
        let bounds = bounds_from(&model);
        let backend_ty = backend_type(&model.backend);
        let accessors = model
            .fields
            .into_iter()
            .map(|f| Accessor::from(f, backend_ty.clone()))
            .collect();

        Self {
            definition,
            new,
            accessors,
            bounds,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ir_gen_does_not_crash() {
        let _ir = Ir::from(Model::mock_u8field());
    }
}
