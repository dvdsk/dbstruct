use proc_macro2::TokenStream;

use crate::model::Model;

pub struct Accessor {
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub returns: syn::Type,
    pub body: syn::Stmt,
}

pub struct Struct {
    pub ident: syn::Ident,
    pub vis: syn::Visibility,
    pub vars: Vec<syn::Field>,
}

pub struct NewMethod {
    pub init_vars: Vec<syn::Local>,
    pub vars: Vec<syn::FieldValue>,
}

pub struct Ir {
    definition: Struct,
    new: NewMethod,
    accessors: Vec<Accessor>,
    bounds: syn::WhereClause,
}

impl From<Model> for Ir {
    fn from(model: Model) -> Self {
        todo!()
    }
}

impl Ir {
    pub fn codegen(self) -> TokenStream {
        todo!()
    }
}
