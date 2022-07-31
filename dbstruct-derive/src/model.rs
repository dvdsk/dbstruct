mod field;
use field::Field;

mod key;
use key::DbKey;
use proc_macro2::TokenStream;

#[derive(Debug)]
pub struct Model {
    fields: Vec<Field>,
    keys: DbKey,
}

impl From<syn::ItemStruct> for Model {
    fn from(input: syn::ItemStruct) -> Self {
        Self {
        keys : DbKey::new(&input.fields).unwrap(),
        fields:  input.fields.into_iter().map(Field::from).collect(),
        }
    }
}

impl Model {
    pub fn into_token_stream(self) -> TokenStream {
        todo!()
    }
}
