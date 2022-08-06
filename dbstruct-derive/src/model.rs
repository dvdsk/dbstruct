mod field;
pub use field::Field;
pub use field::Wrapper;

pub mod key;
use itertools::Itertools;
pub use key::DbKey;
use proc_macro2::Ident;
use syn::Visibility;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("One or more errors occured while analyzing struct fields")]
    Field(Vec<field::Error>),
    #[error(transparent)]
    DbKey(#[from] key::Error),
}

#[derive(Debug)]
pub struct Model {
    pub ident: Ident,
    pub vis: Visibility,
    pub fields: Vec<Field>,
    pub keys: DbKey,
}

impl TryFrom<syn::ItemStruct> for Model {
    type Error = Error;
    fn try_from(input: syn::ItemStruct) -> Result<Self, Self::Error> {
        let keys = DbKey::new(&input.fields)?;

        let (fields, errors): (Vec<_>, Vec<_>) = input
            .fields
            .into_iter()
            .map(Field::try_from)
            .partition_result();

        if !errors.is_empty() {
            return Err(Error::Field(errors));
        }

        Ok(Self {
            vis: input.vis,
            ident: input.ident,
            keys,
            fields,
        })
    }
}

impl Model {
    #[cfg(test)]
    pub fn mock() -> Model {
        let input: syn::ItemStruct = syn::parse_str(
            "        
#[dbstruct::dbstruct]
pub struct Test {
    #[dbstruct(Default)]
    the_field: u8,
}",
        )
        .unwrap();

        Model::try_from(input).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_str;

    #[test]
    fn analyze_model_does_not_crash() {
        let input: syn::ItemStruct = parse_str(
            "        
#[dbstruct::dbstruct]
pub struct Test {
    #[dbstruct(Default)]
    the_field: u8,
}",
        )
        .unwrap();

        let _model = Model::try_from(input).unwrap();
    }
}
