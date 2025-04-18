# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate

## [0.5.0] - 2025-04-04

### Changed
- Updated `embedded-hal` 0.2.5 -> 1.0.0
- Updated `linux-embedded-hal` 0.3 -> 0.4.0
- Updated `embedded-hal-mock` 0.7 -> 0.11.1
- Updated MSRV to Rust 1.62.0

## [0.4.0] - 2021-05-22

### Added
- `SlaveAddr` and `PinFlag` types now implement the traits `Clone`, `Copy` and `PartialEq`.

### Changed
- [breaking-change] Accept pin mask in `read_array()` and `get()` methods by value instead of by reference.

## [0.3.0] - 2019-06-08

### Changed
- [breaking-change] Individual pins now implement
  `embedded_hal::digital::v2::OutputPin` and `embedded_hal::digital::v2::InputPin`
  which can return errors in their `set_high()` and similar methods.
  Previously errors occurred during these operations could not be returned and
  the driver panicked. Now this driver is free from panics.
  See: https://github.com/eldruin/pcf857x-rs/issues/2

## [0.2.0] - 2018-10-20

### Added
- Added method to split a device into structs representing the individual pins
  implementing the `InputPin` and `OutputPin` traits so that it is possible
  to use them transparently as if they were normal I/O pins.
  See: https://github.com/eldruin/pcf857x-rs/issues/1

### Changed
- [breaking-change] Renamed PCF8574 -> Pcf8574, PCF8574A -> Pcf8574a and
  PCF8575 -> Pcf8575 to comply with the Rust naming conventions.

## 0.1.0 - 2018-08-22

This is the initial release to crates.io of the feature-complete driver. There
may be some API changes in the future. All changes will be documented in this
CHANGELOG.

<!-- next-url -->
[Unreleased]: https://github.com/eldruin/pcf857x-rs/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/eldruin/pcf857x-rs/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/eldruin/pcf857x-rs/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/eldruin/pcf857x-rs/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/eldruin/pcf857x-rs/compare/v0.1.0...v0.2.0
