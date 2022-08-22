mod accessor;
mod new_method;
mod struct_def;

pub use accessor::Accessor;
pub use new_method::NewMethod;
pub use struct_def::Struct;
use syn::parse_quote;

use crate::model::Model;

pub struct Ir {
    pub definition: Struct,
    pub new: NewMethod,
    pub accessors: Vec<Accessor>,
    pub bounds: syn::WhereClause,
}

impl Ir {
    pub fn from(model: Model) -> Self {
        let definition = Struct::from(&model);
        let new = NewMethod::from(&model, &definition);
        let bounds: syn::WhereClause = if model.has_vec_field() {
            parse_quote!(where DS: dbstruct::DataStore + dbstruct::traits::data_store::Orderd + std::clone::Clone)
        } else {
            parse_quote!(where DS: dbstruct::DataStore + std::clone::Clone)
        };
        let accessors = model
            .fields
            .into_iter()
            .map(|f| Accessor::from(f))
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
