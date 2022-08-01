mod field;
use field::Field;

mod key;
use key::DbKey;
use proc_macro2::TokenStream;
use proc_macro_error::emit_error;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not determine wrapper for field: {0:?}")]
    WrapperError(#[from] field::Error),
}

#[derive(Debug)]
pub struct Model {
    fields: Vec<Field>,
    keys: DbKey,
}

impl TryFrom<syn::ItemStruct> for Model {
    type Error = Error;
    fn try_from(input: syn::ItemStruct) -> Result<Self, Self::Error> {
        let mut fields = Vec::new();
        for field in input.fields.into_iter().map(Field::try_from) {
            let err = match field {
                Ok(field) => {
                    fields.push(field);
                    continue;
                }
                Err(err) => err,
            };

            // emit_error!()
            todo!("emit error")
        }
        Ok(Self {
            keys: DbKey::new(&input.fields).unwrap(),
            fields,
        })
    }
}

impl Model {
    pub fn into_token_stream(self) -> TokenStream {
        todo!()
    }
}
