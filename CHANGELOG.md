# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
 - the map wrapper (HashMap like structure) now supports `remove`

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
