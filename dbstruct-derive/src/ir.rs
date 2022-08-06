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
        let accessors = model
            .fields
            .into_iter()
            .map(|f| Accessor::from(f, &model.keys))
            .collect();
        let bounds: syn::WhereClause = parse_quote!(where DS: DataStore + Clone);
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
        let _ir = Ir::from(Model::mock());
    }
}
