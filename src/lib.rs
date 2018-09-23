//! This is a platform agnostic Rust driver for the PCF8574, PCF8574A and PCF8575
//! I/O expanders, based on the [`embedded-hal`] traits.
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//!
//! This driver allows you to:
//! - Set all the outputs to `0` or `1` at once.
//! - Read selected inputs.
//! - Set all the outputs repeatedly looping through an array.
//! - Read selected inputs repeatedly filling up an array.
//! - Split the device into individual input/output pins.
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
//! ## Usage examples (see also examples folder)
//!
//! ### Instantiating with the default address
//!
//! Import this crate and an `embedded_hal` implementation, then instantiate
//! the device:
//!
//! ```no_run
//! extern crate linux_embedded_hal as hal;
//! extern crate pcf857x;
//!
//! use hal::I2cdev;
//! use pcf857x::{PCF8574, SlaveAddr};
//!
//! # fn main() {
//! let dev = I2cdev::new("/dev/i2c-1").unwrap();
//! let address = SlaveAddr::default();
//! let mut expander = PCF8574::new(dev, address);
//! # }
//! ```
//!
//! ### Providing an alternative address
//!
//! ```no_run
//! extern crate linux_embedded_hal as hal;
//! extern crate pcf857x;
//!
//! use hal::I2cdev;
//! use pcf857x::{PCF8574, SlaveAddr};
//!
//! # fn main() {
//! let dev = I2cdev::new("/dev/i2c-1").unwrap();
//! let (a2, a1, a0) = (false, false, true);
//! let address = SlaveAddr::Alternative(a2, a1, a0);
//! let mut expander = PCF8574::new(dev, address);
//! # }
//! ```
//!
//! ### Setting the output pins and reading P0 and P7
//!
//! ```no_run
//! extern crate linux_embedded_hal as hal;
//! extern crate pcf857x;
//!
//! use hal::I2cdev;
//! use pcf857x::{PCF8574, SlaveAddr, PinFlag};
//!
//! # fn main() {
//! let dev = I2cdev::new("/dev/i2c-1").unwrap();
//! let address = SlaveAddr::default();
//! let mut expander = PCF8574::new(dev, address);
//! let output_pin_status = 0b1010_1010;
//! expander.set(output_pin_status).unwrap();
//!
//! let mask_of_pins_to_be_read = PinFlag::P0 | PinFlag::P7;
//! let read_input_pin_status = expander.get(&mask_of_pins_to_be_read).unwrap();
//!
//! println!("Input pin status: {}", read_input_pin_status);
//! # }
//! ```
//!
//! ### Splitting device into individual output pins and setting them.
//!
//! ```no_run
//! extern crate linux_embedded_hal as hal;
//! extern crate pcf857x;
//!
//! use hal::I2cdev;
//! use pcf857x::{PCF8574, SlaveAddr, PinFlag, OutputPin};
//!
//! # fn main() {
//! let dev = I2cdev::new("/dev/i2c-1").unwrap();
//! let address = SlaveAddr::default();
//! let expander = PCF8574::new(dev, address);
//! let mut parts = expander.split();
//! parts.p0.set_high();
//! parts.p7.set_low();
//! # }
//! ```

#![deny(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate embedded_hal as hal;
pub use hal::digital::OutputPin;
#[cfg(feature = "unproven")]
pub use hal::digital::InputPin;

/// All possible errors in this crate
#[derive(Debug)]
pub enum Error<E> {
    /// I²C bus error
    I2C(E),
    /// Invalid input data
    InvalidInputData,
    /// Could not acquire device. Maybe it is already acquired.
    CouldNotAcquireDevice
}

/// I/O pin flags, used to select which pins to read in the `get` functions.
/// It is possible to select multiple of them using the binary _or_ operator (`|`).
/// ```
/// # use pcf857x::PinFlag;
/// let pins_to_be_read = PinFlag::P0 | PinFlag::P1;
/// ```
/// Note that P10-17 can only be used with PCF8575 devices.
#[derive(Debug, Clone)]
pub struct PinFlag {
    mask: u16
}

impl PinFlag {
    /// Pin 0
    pub const P0  :  PinFlag = PinFlag { mask:     1 };
    /// Pin 1
    pub const P1  :  PinFlag = PinFlag { mask:     2 };
    /// Pin 2
    pub const P2  :  PinFlag = PinFlag { mask:     4 };
    /// Pin 3
    pub const P3  :  PinFlag = PinFlag { mask:     8 };
    /// Pin 4
    pub const P4  :  PinFlag = PinFlag { mask:    16 };
    /// Pin 5
    pub const P5  :  PinFlag = PinFlag { mask:    32 };
    /// Pin 6
    pub const P6  :  PinFlag = PinFlag { mask:    64 };
    /// Pin 7
    pub const P7  :  PinFlag = PinFlag { mask:   128 };

    /// Pin 10 (only PCF8575)
    pub const P10 :  PinFlag = PinFlag { mask:   256 };
    /// Pin 11 (only PCF8575)
    pub const P11 :  PinFlag = PinFlag { mask:   512 };
    /// Pin 12 (only PCF8575)
    pub const P12 :  PinFlag = PinFlag { mask:  1024 };
    /// Pin 13 (only PCF8575)
    pub const P13 :  PinFlag = PinFlag { mask:  2048 };
    /// Pin 14 (only PCF8575)
    pub const P14 :  PinFlag = PinFlag { mask:  4096 };
    /// Pin 15 (only PCF8575)
    pub const P15 :  PinFlag = PinFlag { mask:  8192 };
    /// Pin 16 (only PCF8575)
    pub const P16 :  PinFlag = PinFlag { mask: 16384 };
    /// Pin 17 (only PCF8575)
    pub const P17 :  PinFlag = PinFlag { mask: 32768 };
}

use core::ops::BitOr;

impl BitOr for PinFlag {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        PinFlag { mask: self.mask | rhs.mask }
    }
}

/// Possible slave addresses
#[derive(Debug)]
pub enum SlaveAddr {
    /// Default slave address
    Default,
    /// Alternative slave address providing bit values for A2, A1 and A0
    Alternative(bool, bool, bool)
}

impl Default for SlaveAddr {
    /// Default slave address
    fn default() -> Self {
        SlaveAddr::Default
    }
}

impl SlaveAddr {
    fn addr(self, default: u8) -> u8 {
        match self {
            SlaveAddr::Default => default,
            SlaveAddr::Alternative(a2, a1, a0) => default           |
                                                  ((a2 as u8) << 2) |
                                                  ((a1 as u8) << 1) |
                                                    a0 as u8
        }
    }
}

mod pins;
pub use pins::{ pcf8574, pcf8575,
                 P0,  P1,  P2,  P3,  P4,  P5,  P6,  P7,
                P10, P11, P12, P13, P14, P15, P16, P17 };
mod devices;
pub use devices::pcf8574::{ PCF8574, PCF8574A };
pub use devices::pcf8575::PCF8575;


#[cfg(test)]
mod tests {
    extern crate embedded_hal_mock as hal;

    use super::*;

    #[test]
    fn can_get_default_address() {
        let addr = SlaveAddr::default();
        assert_eq!(0b010_0000, addr.addr(0b010_0000));
    }

    #[test]
    fn can_generate_alternative_addresses() {
        let default = 0b010_0000;
        assert_eq!(0b010_0000, SlaveAddr::Alternative(false, false, false).addr(default));
        assert_eq!(0b010_0001, SlaveAddr::Alternative(false, false,  true).addr(default));
        assert_eq!(0b010_0010, SlaveAddr::Alternative(false,  true, false).addr(default));
        assert_eq!(0b010_0100, SlaveAddr::Alternative( true, false, false).addr(default));
        assert_eq!(0b010_0111, SlaveAddr::Alternative( true,  true,  true).addr(default));
    }

    #[test]
    fn pin_flags_are_correct() {
        assert_eq!(1,   PinFlag::P0.mask);
        assert_eq!(2,   PinFlag::P1.mask);
        assert_eq!(4,   PinFlag::P2.mask);
        assert_eq!(8,   PinFlag::P3.mask);
        assert_eq!(16,  PinFlag::P4.mask);
        assert_eq!(32,  PinFlag::P5.mask);
        assert_eq!(64,  PinFlag::P6.mask);
        assert_eq!(128, PinFlag::P7.mask);

        assert_eq!(1 << 8,   PinFlag::P10.mask);
        assert_eq!(2 << 8,   PinFlag::P11.mask);
        assert_eq!(4 << 8,   PinFlag::P12.mask);
        assert_eq!(8 << 8,   PinFlag::P13.mask);
        assert_eq!(16 << 8,  PinFlag::P14.mask);
        assert_eq!(32 << 8,  PinFlag::P15.mask);
        assert_eq!(64 << 8,  PinFlag::P16.mask);
        assert_eq!(128 << 8, PinFlag::P17.mask);
    }
}
