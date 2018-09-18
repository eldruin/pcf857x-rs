# Rust PCF857x I/O Expanders Driver [![crates.io](https://img.shields.io/crates/v/pcf857x.svg)](https://crates.io/crates/pcf857x) [![Docs](https://docs.rs/pcf857x/badge.svg)](https://docs.rs/pcf857x)

This is a platform agnostic Rust driver for the PCF8574, PCF8574A and PCF8575 I/O expanders,
based on the [`embedded-hal`](https://github.com/japaric/embedded-hal) traits.

This driver allows you to:
- Set all the outputs to `0` or `1` at once.
- Read selected inputs.
- Set all the outputs repeatedly looping through an array.
- Read selected inputs repeatedly filling up an array.
- Split the device into individual output pins.

## The devices
The devices consist of 8 or 16 quasi-bidirectional ports, I²C-bus interface, three
hardware address inputs and interrupt output. The quasi-bidirectional port can be
independently assigned as an input to monitor interrupt status or keypads, or as an
output to activate indicator devices such as LEDs.

The active LOW open-drain interrupt output (INT) can be connected to the interrupt logic
of the microcontroller and is activated when any input state differs from its corresponding 
input port register state.

Datasheets:
- [PCF8574 / PCF8574A](https://www.nxp.com/docs/en/data-sheet/PCF8574_PCF8574A.pdf)
- [PCF8575](https://www.nxp.com/documents/data_sheet/PCF8575.pdf)

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT) at your option.

### Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

