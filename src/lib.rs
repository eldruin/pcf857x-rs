//! This is a platform agnostic Rust driver for the PCF8574, PCF8574A and PCF8575
//! I/O expanders, based on the [`embedded-hal`] traits.
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//!
//! This driver allows you to:
//! - Set all the outputs to `0` or `1` at once. See `set()`.
//! - Read selected inputs. See `get()`.
//! - Set all the outputs repeatedly looping through an array. See `write_array()`.
//! - Read selected inputs repeatedly filling up an array. See `read_array()`.
//! - Split the device into individual input/output pins. See `split()`.
//!
//! ## The devices
//! The devices consist of 8 or 16 quasi-bidirectional ports, I²C-bus interface, three
//! hardware address inputs and interrupt output. The quasi-bidirectional port can be
//! independently assigned as an input to monitor interrupt status or keypads, or as an
//! output to activate indicator devices such as LEDs.
//!
//! The active LOW open-drain interrupt output (INT) can be connected to the interrupt logic
//! of the microcontroller and is activated when any input state differs from its corresponding
//! input port register state.
//!
//! Datasheets:
//! - [PCF8574 / PCF8574A](https://www.nxp.com/docs/en/data-sheet/PCF8574_PCF8574A.pdf)
//! - [PCF8575](https://www.nxp.com/documents/data_sheet/PCF8575.pdf)
//!
//! ## Splitting the device into individual input/output pins
//!
//! By calling `split()` on the device it is possible to get a structure holding the
//! individual pins as separate elements. These pins implement the `OutputPin` and
//! `InputPin` traits (the latter only if activating the `unproven` feature).
//! This way it is possible to use the pins transparently as normal I/O pins regardless
//! of the fact that an I/O expander is connected in between.
//! You can therefore also pass them to code expecting an `OutputPin` or `InputPin`.
//!
//! However, you need to keep the device you split alive (lifetime annotations have
//! put in place for Rust to enforce this).
//!
//! For each operation done on an input/output pin, a `read` or `write` will be done
//! through I2C for all the pins, using a cached value for the rest of pins not being
//! operated on. This should all be transparent to the user but if operating on more
//! than one pin at once, the `set` and `get` methods will be faster.
//! Similarly, if several pins must be changed/read at the same time, the `set` and
//! `get` methods would be the correct choice.
//!
//! At the moment, no mutex has been implemented for the individual pin access.
//!
//! ## Usage examples (see also examples folder)
//!
//! Please find additional examples using hardware in this repository: [driver-examples]
//!
//! [driver-examples]: https://github.com/eldruin/driver-examples
//!
//! ### Instantiating with the default address
//!
//! Import this crate and an `embedded_hal` implementation, then instantiate
//! the device:
//!
//! ```no_run
//! use linux_embedded_hal::I2cdev;
//! use pcf857x::{ Pcf8574, SlaveAddr };
//!
//! let dev = I2cdev::new("/dev/i2c-1").unwrap();
//! let address = SlaveAddr::default();
//! let mut expander = Pcf8574::new(dev, address);
//! ```
//!
//! ### Providing an alternative address
//!
//! ```no_run
//! use linux_embedded_hal::I2cdev;
//! use pcf857x::{ Pcf8574, SlaveAddr };
//!
//! let dev = I2cdev::new("/dev/i2c-1").unwrap();
//! let (a2, a1, a0) = (false, false, true);
//! let address = SlaveAddr::Alternative(a2, a1, a0);
//! let mut expander = Pcf8574::new(dev, address);
//! ```
//!
//! ### Setting the output pins and reading P0 and P7
//!
//! ```no_run
//! use linux_embedded_hal::I2cdev;
//! use pcf857x::{ Pcf8574, SlaveAddr, PinFlag };
//!
//! let dev = I2cdev::new("/dev/i2c-1").unwrap();
//! let address = SlaveAddr::default();
//! let mut expander = Pcf8574::new(dev, address);
//! let output_pin_status = 0b1010_1010;
//! expander.set(output_pin_status).unwrap();
//!
//! let mask_of_pins_to_be_read = PinFlag::P0 | PinFlag::P7;
//! let read_input_pin_status = expander.get(mask_of_pins_to_be_read).unwrap();
//!
//! println!("Input pin status: {}", read_input_pin_status);
//! ```
//!
//! ### Splitting device into individual input/output pins and setting them.
//!
//! ```no_run
//! use linux_embedded_hal::I2cdev;
//! use pcf857x::{ Pcf8574, SlaveAddr, PinFlag, OutputPin };
//!
//! let dev = I2cdev::new("/dev/i2c-1").unwrap();
//! let address = SlaveAddr::default();
//! let expander = Pcf8574::new(dev, address);
//! let mut parts = expander.split();
//! parts.p0.set_high().unwrap();
//! parts.p7.set_low().unwrap();
//! ```
//!
//! ### Splitting device into individual input/output pins and reading them.
//!
//! Only available if compiling with the "`unproven`" feature
//!
//! ```no_run
//! use linux_embedded_hal::I2cdev;
//! use pcf857x::{ Pcf8574, SlaveAddr, PinFlag };
//! #[cfg(feature="unproven")]
//! use pcf857x::InputPin;
//!
//! let dev = I2cdev::new("/dev/i2c-1").unwrap();
//! let address = SlaveAddr::default();
//! let expander = Pcf8574::new(dev, address);
//! let mut parts = expander.split();
//! #[cfg(feature="unproven")]
//! {
//!     let is_input_p0_low = parts.p0.is_low().unwrap();
//!     let is_input_p2_low = parts.p2.is_low().unwrap();
//! }
//! ```

#![deny(unsafe_code)]
#![deny(missing_docs)]
#![no_std]

pub use embedded_hal::digital::InputPin;
pub use embedded_hal::digital::OutputPin;

/// All possible errors in this crate
#[derive(Debug)]
pub enum Error<E> {
    /// I²C bus error
    I2C(E),
    /// Invalid input data
    InvalidInputData,
    /// Could not acquire device. Maybe it is already acquired.
    CouldNotAcquireDevice,
}

impl<E: core::fmt::Debug> embedded_hal::digital::Error for Error<E> {
    fn kind(&self) -> embedded_hal::digital::ErrorKind {
        embedded_hal::digital::ErrorKind::Other
    }
}

mod slave_addr;
pub use crate::slave_addr::SlaveAddr;
mod pin_flag;
pub use crate::pin_flag::PinFlag;
mod split_pins;
pub use crate::split_pins::{
    pcf8574, pcf8575, P0, P1, P10, P11, P12, P13, P14, P15, P16, P17, P2, P3, P4, P5, P6, P7,
};
mod devices;
pub use crate::devices::pcf8574::{Pcf8574, Pcf8574a};
pub use crate::devices::pcf8575::Pcf8575;
