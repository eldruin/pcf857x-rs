//! This is a platform agnostic Rust driver for the PCF8574, PCF8574A and PCF8575
//! I/O expanders, based on the [`embedded-hal`] traits.
//!
//! [`embedded-hal`]: https://github.com/japaric/embedded-hal
//!
//! This driver allows you to:
//! - Set all the outputs to `0` or `1` at once.
//! - Read selected inputs.
//! - Set all the outputs repeatedly looping through an array.
//! - Read selected inputs repeatedly filling up an array.
//! - Split the device into individual output pins.
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
//! use hal::{I2cdev};
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
//! use hal::{I2cdev};
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
//! use hal::{I2cdev};
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
//! use hal::{I2cdev};
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
#![no_std]

extern crate embedded_hal as hal;
use hal::blocking::i2c::Write;
pub use hal::digital::OutputPin;

#[cfg(feature = "std")]
use std::cell;

#[cfg(not(feature = "std"))]
use core::cell;

#[cfg(feature = "std")]
use std::marker;

#[cfg(not(feature = "std"))]
use core::marker;

use marker::PhantomData;

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

macro_rules! pins {
    ( $( $PX:ident ),+ ) => {
        $(  /// Pin
            pub struct $PX<'a, IC: 'a, E>(&'a IC, PhantomData<E>);
        )*
    }
}
pins!( P0,  P1,  P2,  P3,  P4,  P5,  P6,  P7,
      P10, P11, P12, P13, P14, P15, P16, P17);

macro_rules! parts {
    ( $( $px:ident, $PX:ident ),+ ) => {
        $(
            use super::$PX;
        )*
        /// Pins
        pub struct Parts<'a, IC:'a, E> {
            $(
                /// Pin
                pub $px: $PX<'a, IC, E>,
            )*
        }

        use super::PhantomData;
        impl<'a, IC:'a, E> Parts<'a, IC, E> {
            pub(crate) fn new(ic: &'a IC) -> Self {
                Parts {
                    $(
                        $px: $PX(&ic, PhantomData),
                    )*
                }
            }
        }
    }
}

/// Module containing structures specific to PCF8574 and PCF8574A
pub mod pcf8574 {
    parts!(p0, P0, p1, P1, p2, P2, p3, P3, p4, P4, p5, P5, p6, P6, p7, P7);
}

macro_rules! pcf8574 {
    ( $device_name:ident, $device_data_name:ident, $default_address:expr ) => {
        /// Device driver
        #[derive(Debug, Default)]
        pub struct $device_name<I2C> {
            /// Data
            data: cell::RefCell<$device_data_name<I2C>>
        }

        #[derive(Debug, Default)]
        struct $device_data_name<I2C> {
            /// The concrete I²C device implementation.
            i2c: I2C,
            /// The I²C device address.
            address: u8,
            /// Last status set to output pins, used to conserve its status while doing a read.
            last_set_mask: u8,
        }

        impl<I2C, E> $device_name<I2C>
        where
            I2C: Write<Error = E>
        {
            /// Create new instance of the device
            pub fn new(i2c: I2C, address: SlaveAddr) -> Self {
                let data = $device_data_name {
                    i2c,
                    address: address.addr($default_address),
                    last_set_mask: 0
                };
                $device_name {
                    data: cell::RefCell::new(data)
                }
            }

            /// Destroy driver instance, return I²C bus instance.
            pub fn destroy(self) -> I2C {
                self.data.into_inner().i2c
            }

            fn acquire_device(&self) -> Result<cell::RefMut<$device_data_name<I2C>>, Error<E>> {
                self.data.try_borrow_mut().map_err(|_| Error::CouldNotAcquireDevice)
            }

            /// Set the status of all I/O pins.
            pub fn set(&mut self, bits: u8) -> Result<(), Error<E>> {
                let mut dev = self.acquire_device()?;
                let address = dev.address;
                dev.i2c
                    .write(address, &[bits])
                    .map_err(Error::I2C)?;
                dev.last_set_mask = bits;
                Ok(())
            }

            /// Set the status of all I/O pins repeatedly by looping through each array element
            pub fn write_array(&mut self, data: &[u8]) -> Result<(), Error<E>> {
                if let Some(last) = data.last() {
                    let mut dev = self.acquire_device()?;
                    let address = dev.address;
                    dev.i2c
                        .write(address, &data)
                        .map_err(Error::I2C)?;
                    dev.last_set_mask = *last;
                }
                Ok(())
            }

            /// Split device into individual pins
            pub fn split<'a>(&'a self) -> pcf8574::Parts<'a, $device_name<I2C>, E> {
                pcf8574::Parts::new(&self)
            }
        }

        impl<I2C, E> $device_name<I2C>
        where
            I2C: hal::blocking::i2c::Read<Error = E> + Write<Error = E>
        {
            /// Get the status of the selected I/O pins.
            /// The mask of the pins to be read can be created with a combination of
            /// `PinFlag::P0` to `PinFlag::P7`.
            pub fn get(&mut self, mask: &PinFlag) -> Result<u8, Error<E>> {
                if (mask.mask >> 8) != 0 {
                    return Err(Error::InvalidInputData);
                }
                let mut dev = self.acquire_device()?;
                let mask = mask.mask as u8 | dev.last_set_mask;
                let address = dev.address;
                // configure selected pins as inputs
                dev.i2c
                    .write(address, &[mask])
                    .map_err(Error::I2C)?;

                let mut bits = [0];
                dev.i2c
                    .read(address, &mut bits)
                    .map_err(Error::I2C).and(Ok(bits[0]))
            }

            /// Get the status of the selected I/O pins repeatedly and put them in the
            /// provided array.
            /// The mask of the pins to be read can be created with a combination of
            /// `PinFlag::P0` to `PinFlag::P7`.
            pub fn read_array(&mut self, mask: &PinFlag, mut data: &mut [u8]) -> Result<(), Error<E>> {
                if !data.is_empty() {
                    if (mask.mask >> 8) != 0 {
                       return Err(Error::InvalidInputData);
                    }
                    let mut dev = self.acquire_device()?;
                    let mask = mask.mask as u8 | dev.last_set_mask;
                    let address = dev.address;
                    // configure selected pins as inputs
                    dev.i2c
                        .write(address, &[mask])
                        .map_err(Error::I2C)?;

                    dev.i2c
                        .read(address, &mut data)
                        .map_err(Error::I2C)?;
                }
                Ok(())
            }
        }

    };
}

pcf8574!(PCF8574,  PCF8574Data,  0b010_0000);
pcf8574!(PCF8574A, PCF8574AData, 0b011_1000);


/// PCF8575 device driver
#[derive(Debug, Default)]
pub struct PCF8575<I2C> {
    /// The concrete I²C device implementation.
    i2c: I2C,
    /// The I²C device address.
    address: u8,
    /// Last status set to output pins, used to conserve its status while doing a read.
    last_set_mask: u16,
}

impl<I2C, E> PCF8575<I2C>
where
    I2C: Write<Error = E>
{
    /// Create new instance of the PCF8575 device
    pub fn new(i2c: I2C, address: SlaveAddr) -> Self {
        PCF8575 {
            i2c,
            address: address.addr(0b010_0000),
            last_set_mask: 0
        }
    }

    /// Destroy driver instance, return I²C bus instance.
    pub fn destroy(self) -> I2C {
        self.i2c
    }

    /// Set the status of all I/O pins.
    pub fn set(&mut self, bits: u16) -> Result<(), Error<E>> {
        self.i2c
            .write(self.address, &u16_to_u8_array(bits)[..])
            .map_err(Error::I2C)?;
        self.last_set_mask = bits;
        Ok(())
    }

    /// Set the status of all I/O pins repeatedly by looping through each array element.
    /// The even elements correspond to the status of P0-P7 and the odd ones P10-P17.
    /// The number of elements in the data must be even.
    pub fn write_array(&mut self, data: &[u8]) -> Result<(), Error<E>> {
        if !data.is_empty() {
            if data.len() % 2 != 0 {
                return Err(Error::InvalidInputData);
            }
            self.i2c
                .write(self.address, &data)
                .map_err(Error::I2C)?;
            self.last_set_mask = ((data[data.len()-1] as u16) << 8) | data[data.len()-2] as u16;
        }
        Ok(())
    }
}

impl<I2C, E> PCF8575<I2C>
where
    I2C: hal::blocking::i2c::Read<Error = E> + Write<Error = E>
{
    /// Get the status of the selected I/O pins.
    /// The mask of the pins to be read can be created with a combination of
    /// `PinFlag::P0` to `PinFlag::P17`.
    pub fn get(&mut self, mask: &PinFlag) -> Result<u16, Error<E>> {
        let mask = mask.mask | self.last_set_mask;
        // configure selected pins as inputs
        self.i2c
            .write(self.address, &u16_to_u8_array(mask)[..])
            .map_err(Error::I2C)?;

        let mut bits = [0; 2];
        self.i2c
            .read(self.address, &mut bits)
            .map_err(Error::I2C).and(Ok(u8_array_to_u16(bits)))
    }

    /// Get the status of the selected I/O pins repeatedly and put them in the
    /// provided array.
    /// The mask of the pins to be read can be created with a combination of
    /// `PinFlag::P0` to `PinFlag::P17`.
    /// The even elements correspond to the status of P0-P7 and the odd ones P10-P17.
    /// The number of elements in the data must be even.
    pub fn read_array(&mut self, mask: &PinFlag, mut data: &mut [u8]) -> Result<(), Error<E>> {
        if !data.is_empty() {
            if data.len() % 2 != 0 {
                return Err(Error::InvalidInputData);
            }
            let mask = mask.mask | self.last_set_mask;
            // configure selected pins as inputs
            self.i2c
                .write(self.address, &u16_to_u8_array(mask))
                .map_err(Error::I2C)?;

            self.i2c
                .read(self.address, &mut data)
                .map_err(Error::I2C)?;
        }
        Ok(())
    }
}

fn u16_to_u8_array(input: u16) -> [u8; 2] {
    [input as u8, (input >> 8) as u8]
}

fn u8_array_to_u16(input: [u8; 2]) -> u16 {
    input[0] as u16 | ((input[1] as u16) << 8)
}

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

    #[test]
    fn can_convert_u16_to_u8_array() {
        assert_eq!([0xCD, 0xAB], u16_to_u8_array(0xABCD));
    }

    #[test]
    fn can_convert_u8_array_to_u16() {
        assert_eq!(0xABCD, u8_array_to_u16([0xCD, 0xAB]));
    }

    macro_rules! pcf8574_tests {
        ($device_name:ident, $test_mod_name:ident, $default_address:expr) => {
            mod $test_mod_name {
                use super::*;
                fn setup<'a>(data: &'a[u8]) -> $device_name<hal::I2cMock<'a>> {
                    let mut dev = hal::I2cMock::new();
                    dev.set_read_data(&data);
                    $device_name::new(dev, SlaveAddr::default())
                }

                fn check_sent_data(expander: $device_name<hal::I2cMock>, data: &[u8]) {
                    let dev = expander.destroy();
                    assert_eq!(dev.get_last_address(), Some($default_address));
                    assert_eq!(dev.get_write_data(), &data[..]);
                }

                #[test]
                fn can_read_pins() {
                    let mut expander = setup(&[0x01]);
                    let mask = PinFlag::P0 | PinFlag::P7;
                    let status = expander.get(&mask).unwrap();
                    check_sent_data(expander, &[mask.mask as u8]);
                    assert_eq!(0x01, status);
                }

                #[test]
                fn read_conserves_output_high_pins() {
                    let mut expander = setup(&[0x01]);
                    let write_status = 0b0101_1010;
                    expander.set(write_status).unwrap();
                    let mask = PinFlag::P0 | PinFlag::P7;
                    let read_status = expander.get(&mask).unwrap();
                    check_sent_data(expander, &[mask.mask as u8 | write_status]);
                    assert_eq!(0x01, read_status);
                }

                #[test]
                fn can_read_multiple_words() {
                    let mut data = [0; 2];
                    let mut expander = setup(&[0xAB, 0xCD]);
                    let mask = PinFlag::P0 | PinFlag::P7;
                    expander.read_array(&mask, &mut data).unwrap();
                    check_sent_data(expander, &[mask.mask as u8]);
                    assert_eq!([0xAB, 0xCD], data);
                }


                #[test]
                fn reading_multiple_words_conserves_high_pins() {
                    let mut expander = setup(&[0xAB, 0xCD]);
                    let write_status = 0b0101_1010;
                    expander.set(write_status).unwrap();
                    let mut read_data = [0; 2];
                    let mask = PinFlag::P0 | PinFlag::P7;
                    expander.read_array(&mask, &mut read_data).unwrap();
                    check_sent_data(expander, &[mask.mask as u8 | write_status]);
                    assert_eq!([0xAB, 0xCD], read_data);
                }
            }
        }
    }

    pcf8574_tests!(PCF8574,  pcf8574_tests,  0b010_0000);
    pcf8574_tests!(PCF8574A, pcf8574a_tests, 0b011_1000);

    mod pcf8575_tests {
        use super::*;
        fn setup<'a>(data: &'a[u8]) -> PCF8575<hal::I2cMock<'a>> {
            let mut dev = hal::I2cMock::new();
            dev.set_read_data(&data);
            PCF8575::new(dev, SlaveAddr::default())
        }

        fn check_sent_data(expander: PCF8575<hal::I2cMock>, data: &[u8]) {
            let dev = expander.destroy();
            assert_eq!(dev.get_last_address(), Some(0b010_0000));
            assert_eq!(dev.get_write_data(), &data[..]);
        }

        #[test]
        fn can_read_pins() {
            let mut expander = setup(&[0x00, 0x01]);
            let mask = PinFlag::P0 | PinFlag::P17;
            let status = expander.get(&mask).unwrap();
            check_sent_data(expander, &u16_to_u8_array(mask.mask));
            assert_eq!(0x0100, status);
        }

        #[test]
        fn read_conserves_output_high_pins() {
            let mut expander = setup(&[0x00, 0x01]);
            let write_status = 0b0101_0101_0101_0101;
            expander.set(write_status).unwrap();
            let mask = PinFlag::P0 | PinFlag::P17;
            let read_status = expander.get(&mask).unwrap();
            check_sent_data(expander, &u16_to_u8_array(mask.mask | write_status));
            assert_eq!(0x0100, read_status);
        }

        #[test]
        fn can_read_multiple_words() {
            let mut data = [0; 2];
            let mut expander = setup(&[0xAB, 0xCD]);
            let mask = PinFlag::P0 | PinFlag::P17;
            expander.read_array(&mask, &mut data).unwrap();
            check_sent_data(expander, &u16_to_u8_array(mask.mask));
            assert_eq!([0xAB, 0xCD], data);
        }


        #[test]
        fn reading_multiple_words_conserves_high_pins() {
            let mut expander = setup(&[0xAB, 0xCD]);
            let write_status = 0b0101_1010;
            expander.set(write_status).unwrap();
            let mut read_data = [0; 2];
            let mask = PinFlag::P0 | PinFlag::P17;
            expander.read_array(&mask, &mut read_data).unwrap();
            check_sent_data(expander, &u16_to_u8_array(mask.mask | write_status));
            assert_eq!([0xAB, 0xCD], read_data);
        }
    }
}

