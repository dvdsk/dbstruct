use proc_macro_error::proc_macro_error;
use syn::parse_macro_input;

mod model;
use model::Model;
mod ir;
use ir::Ir;
// mod methods;

#[proc_macro_attribute]
#[proc_macro_error]
pub fn dbstruct(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as syn::ItemStruct);
    let model = Model::try_from(input).unwrap();
    let ir = Ir::from(model);
    let code = ir.codegen();
    code.into()
}

// fn tree_name(struct_name: &syn::Ident, fields: &syn::Fields) -> String {
//     let mut res = String::new();
//     res.push_str(&struct_name.to_string());
//     for field in fields {
//         res.push(',');
//         let field_ident = field.ident.as_ref().unwrap().to_string();
//         res.push_str(&field_ident);
//     }
//     res
// }

// fn generate_struct(
//     input: ItemStruct,
// ) -> TokenStream {
//     let keys = DbKey::new(&input.fields).unwrap();
//     let fields: Vec<_> = input.fields
//         .iter()
//         .map(|k| Field::from)
//         .collect();
//     let tree_name = tree_name(&input.ident, &input.fields);
//
//     quote! {
//         // struct #input.ident {
//         //     tree: sled::Tree,
//         // }
//         //
//         // impl std::fmt::Debug for #input.ident {
//         //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
//         //         write!(f, "dbstruct")
//         //     }
//         // }
//         //
//         // impl #input.ident {
//         //     pub fn test() -> Result<Self, dbstruct::Error> {
//         //         let db = sled::Config::default()
//         //             .temporary(true)
//         //             .open()
//         //             .unwrap();
//         //         Self::open_tree(db)
//         //     }
//         //
//         //     pub fn open_db(path: impl std::convert::AsRef<std::path::Path>) -> Result<Self, dbstruct::Error> {
//         //         let db = sled::Config::default()
//         //             .path(path)
//         //             .open()
//         //             .unwrap();
//         //         Self::open_tree(db)
//         //     }
//         //     pub fn open_tree(db: sled::Db) -> Result<Self, dbstruct::Error> {
//         //         let tree = db.open_tree(#tree_name).unwrap();
//         //         Ok(Self { tree })
//         //     }
//         //
//         //     #(#field_methods)*
//         //
//         // }
//     }
// }
