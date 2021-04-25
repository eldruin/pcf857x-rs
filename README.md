# Rust PCF857x I/O Expanders Driver

[![crates.io](https://img.shields.io/crates/v/pcf857x.svg)](https://crates.io/crates/pcf857x)
[![Docs](https://docs.rs/pcf857x/badge.svg)](https://docs.rs/pcf857x)
[![Build Status](https://github.com/eldruin/pcf857x-rs/workflows/Build/badge.svg)](https://github.com/eldruin/pcf857x-rs/actions?query=workflow%3ABuild)
[![Coverage Status](https://coveralls.io/repos/github/eldruin/pcf857x-rs/badge.svg?branch=master)](https://coveralls.io/github/eldruin/pcf857x-rs?branch=master)
![Maintenance Intention](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)

This is a platform agnostic Rust driver for the PCF8574, PCF8574A and PCF8575 I2C I/O expanders,
based on the [`embedded-hal`] traits.

This driver allows you to:
- Set all the outputs to `0` or `1` at once. See `set()`.
- Read selected inputs. See `get()`.
- Set all the outputs repeatedly looping through an array. See `write_array()`.
- Read selected inputs repeatedly filling up an array. See `read_array()`.
- Split the device into individual input/output pins. See `split()`.

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

## Usage

Please find additional examples using hardware in this repository: [driver-examples]

[driver-examples]: https://github.com/eldruin/driver-examples

```rust
extern crate embedded_hal;
extern crate linux_embedded_hal;
extern crate pcf857x;

use linux_embedded_hal::I2cdev;
use pcf857x::{Pcf8574, PinFlag, SlaveAddr};

fn main() {
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let address = SlaveAddr::default();
    let mut expander = Pcf8574::new(dev, address);
    let output_pin_status = 0b1010_1010;
    expander.set(output_pin_status).unwrap();

    let pins_to_be_read = PinFlag::P0 | PinFlag::P7;
    let status = expander.get(&pins_to_be_read).unwrap();

    println!("Input pin status: {}", status);
}
```

## Support

For questions, issues, feature requests, and other changes, please file an
[issue in the github project](https://github.com/eldruin/pcf857x-rs/issues).

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

[`embedded-hal`]: https://github.com/rust-embedded/embedded-hal