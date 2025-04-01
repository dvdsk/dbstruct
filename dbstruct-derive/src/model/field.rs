mod wrapper;
pub use wrapper::Error;
pub use wrapper::Wrapper;

use super::DbKey;

#[derive(Debug)]
pub struct Field {
    pub ident: syn::Ident,
    pub vis: syn::Visibility,
    pub wrapper: Wrapper,
    pub key: u8,
}

impl Field {
    pub fn analyze(mut field: syn::Field, keys: &DbKey) -> Result<Self, Error> {
        let wrapper = Wrapper::try_from(&mut field.attrs, field.ty)?;
        let ident = field
            .ident
            .expect("every struct field should have an Ident");
        let key = keys.prefix(&ident);

        Ok(Self {
            ident,
            vis: field.vis,
            wrapper,
            key,
        })
    }
}
