mod wrapper;
use wrapper::Wrapper;

#[derive(Debug)]
pub struct Field {
    ident: syn::Ident,
    vis: syn::Visibility,
    wrapper: Wrapper,
}

impl From<syn::Field> for Field {
    fn from(mut field: syn::Field) -> Self {
        let wrapper = Wrapper::try_from(&mut field.attrs, field.ty);
        todo!("error handling for wrapper")
        // Self {
        //     ident: field
        //         .ident
        //         .expect("every struct field should have an Ident"),
        //     vis: field.vis,
        //     wrapper, 
        // }
    }
}
