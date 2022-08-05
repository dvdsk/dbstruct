mod field;
pub use field::Field;
pub use field::Wrapper;

mod key;
pub use key::DbKey;
use proc_macro2::Ident;
use proc_macro_error::emit_error;
use syn::Visibility;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not determine wrapper for field: {0:?}")]
    WrapperError(#[from] field::Error),
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
        let keys = DbKey::new(&input.fields).unwrap();

        let mut fields = Vec::new();
        for field in input.fields.into_iter().map(Field::try_from) {
            let err = match field {
                Ok(field) => {
                    fields.push(field);
                    continue;
                }
                Err(err) => err,
            };

            emit_error!(err.span(), err);
        }
        Ok(Self {
            vis: input.vis,
            ident: input.ident,
            keys,
            fields,
        })
    }
}
