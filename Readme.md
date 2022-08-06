**This is an early release, the API is unstable and might still change in**

Create a typed database by defining a struct. *dbstruct* builds on top of anything that implements `dbstruct::DataStore` or `dbstruct::BytesStore` and provides a typed API similar to the standard library. An implementation for [sled](https://crates.io/crates/sled) is provided. 

*dbstruct* is ideal when:
- writing a simple app that needs some fast data storage.
- quickly getting a storage layer done when prototyping a system that you can later replace.
