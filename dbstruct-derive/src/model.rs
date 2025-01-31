mod attribute;
pub mod backend;
mod field;
pub mod key;

pub use field::Field;
pub use field::Wrapper;

use itertools::Itertools;
pub use key::DbKey;
use proc_macro2::Ident;
use syn::Visibility;

use self::backend::Backend;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("One or more errors occured while analyzing struct fields")]
    Field(Vec<field::Error>),
    #[error(transparent)]
    DbKey(#[from] key::Error),
    #[error(transparent)]
    Attribute(#[from] attribute::Error),
    #[error(transparent)]
    Backend(#[from] backend::Error),
}

#[derive(Debug)]
pub struct Model {
    pub ident: Ident,
    pub vis: Visibility,
    pub fields: Vec<Field>,
    pub backend: Backend,
}

impl Model {
    pub fn try_from(input: syn::ItemStruct, attr: proc_macro2::TokenStream) -> Result<Self, Error> {
        let keys = DbKey::new(&input.fields)?;

        let (fields, errors): (Vec<_>, Vec<_>) = input
            .fields
            .into_iter()
            .map(|f| Field::analyze(f, &keys))
            .partition_result();

        if !errors.is_empty() {
            return Err(Error::Field(errors));
        }

        let options = attribute::parse(attr)?;
        let backend = Backend::try_from(&options, &fields)?;

        Ok(Self {
            vis: input.vis,
            ident: input.ident,
            fields,
            backend,
        })
    }
}

#[cfg(test)]
use std::str::FromStr;

#[cfg(test)]
impl Model {
    pub fn mock_vec() -> Model {
        let input_attr = proc_macro2::TokenStream::from_str("db=sled").unwrap();
        let input_struct: syn::ItemStruct = syn::parse_str(
            "        
pub struct Test {
    the_field: Vec<u8>,
}",
        )
        .unwrap();

        Model::try_from(input_struct, input_attr).unwrap()
    }

    pub fn mock_u8field() -> Model {
        let input_attr = proc_macro2::TokenStream::from_str("db=sled").unwrap();
        let input_struct: syn::ItemStruct = syn::parse_str(
            "        
pub struct Test {
    #[dbstruct(Default)]
    the_field: u8,
}",
        )
        .unwrap();

        Model::try_from(input_struct, input_attr).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use syn::parse_str;

    #[test]
    fn analyze_model_does_not_crash() {
        let input_attr = proc_macro2::TokenStream::from_str("db=sled").unwrap();
        let input_struct: syn::ItemStruct = parse_str(
            "        
pub struct Test {
    #[dbstruct(Default)]
    the_field: u8,
}",
        )
        .unwrap();

        let _model = Model::try_from(input_struct, input_attr).unwrap();
    }

    mod backend {
        use super::*;

        #[test]
        fn sled() {
            let input_attr = proc_macro2::TokenStream::from_str("db=sled").unwrap();
            let input_struct: syn::ItemStruct = parse_str(
                "        
pub struct Test {
    #[dbstruct(Default)]
    the_field: u8,
}",
            )
            .unwrap();

            let model = Model::try_from(input_struct, input_attr).unwrap();
            assert!(matches!(model.backend, Backend::Sled));
        }

        #[test]
        fn none() {
            let input_attr = proc_macro2::TokenStream::from_str("db=trait").unwrap();
            let input_struct: syn::ItemStruct = parse_str(
                "        
pub struct Test {
    #[dbstruct(Default)]
    the_field: u8,
}",
            )
            .unwrap();

            let model = Model::try_from(input_struct, input_attr).unwrap();
            assert!(matches!(model.backend, Backend::Trait { .. }));
        }
    }
}
