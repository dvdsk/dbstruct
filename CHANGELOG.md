# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
## Added
 - `open_db` and `open_tree` for structs backed by `sled`
 - `Option` fields without default expr can now be set to `None`
 - `Option::is_some` and `Option::is_none`
 
## Changed
 - **Breaking:** structs backed by `sled` must now be opened using `open_path`
 - **Breaking:** `Option` fields without default expr now take Some in `.set()`
 
## Fixed
 - Generated structs are is `Send` again
 

## [0.6.0] - 2025-04-02

### Added
 - Wrappers can now be debug printed
 - Collection wrappers now have an `iter()` method
 - Adds `VecDeque` collection

## Changed
 - **Breaking:** Encoding format has changed, any data created with older
   versions cannot be loaded by this version of `dbstruct`!
 - **Breaking:** The generated struct is no longer `Sync`.

## [0.5.0] - 2025-02-01

### Added
 - Examples added to the documentation of all wrapper member functions
 - The map wrapper (`HashMap` like structure) now supports `remove`
 - The `vec` wrapper now supports `clear`

### Changed
 - **Breaking:** `TryExtend` trait removed in favor of member functions `try_extend`
 - **Breaking:** `try_extend` now works on borrowed values instead of owned.
 - most wrapper functions now accept borrowed items similar to how std
   `HashMap` does
 - returned errors are now clearly generic over the underlying database error
   type

### Fixed
 - can no longer panic in vec/map iterator.

## [0.4.1] - 2023-08-18

### Added
 - implements `TryExtend` for iterator borrowing items

### Fixed
 - crash when database errors while inserting into the map wrapper

## [0.4.0] - 2023-08-17

### Added
 - adds `TryExtend` that mirrors the std's Extend trait
 - implements `TryExtend` for map and `vec` wrapper

### Changed
 - `wrapper::Vec::push` now takes a reference to the item to be pushed

## [0.3.0] - 2023-08-12

### Added
 - adds iterator methods to the map wrapper
 - adds `BTreeMap` backend

### Deprecated
 - `HashMap` test backend (use new `BTreeMap` backend)

## [0.2.1] - 2023-08-02

### Changed
 - updates dbstruct-derive dependency

## [0.2.0] - 2023-08-02

### Added
 - `IntoIterator` implemented for the `Vec` wrapper 

## [0.1.1] - 2023-08-01

### Changed
 - Updates dependencies
