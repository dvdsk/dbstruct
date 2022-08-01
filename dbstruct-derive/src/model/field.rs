mod wrapper;
use wrapper::Wrapper;
pub use wrapper::Error;

#[derive(Debug)]
pub struct Field {
    ident: syn::Ident,
    vis: syn::Visibility,
    wrapper: Wrapper,
}

impl TryFrom<syn::Field> for Field {
    type Error = Error;
    fn try_from(mut field: syn::Field) -> Result<Self, Error> {
        let wrapper = Wrapper::try_from(&mut field.attrs, field.ty)?;

        Ok(Self {
            ident: field
                .ident
                .expect("every struct field should have an Ident"),
            vis: field.vis,
            wrapper, 
        })
    }
}
