// pub struct Test<DS: dbstruct::DataStore + std::clone::Clone> {
//     ds: DS,
// }
//
// impl<DS> Test<DS>
// where
//     DS: dbstruct::DataStore + std::clone::Clone,
// {
//     pub fn new(ds: DS) -> Result<Self, dbstruct::Error<DS::Error>> {
//         Ok(Self {
//             ds,
//             ds: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
//         })
//     }
//     fn the_field(&self) -> dbstruct::wrappers::DefaultTrait<u8, DS> {
//         dbstruct::wrappers::DefaultTrait::new(self.ds.clone(), 0u8)
//     }
// }
//
// fn main() {}
