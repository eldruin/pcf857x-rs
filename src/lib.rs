//! This is a platform agnostic Rust driver for the PCF8574, PCF8574A and PCF8575
//! I/O expanders, based on the [`embedded-hal`] traits.
//! [`embedded-hal`]: https://github.com/japaric/embedded-hal
//!
//! This driver allows you to:
//! - Set all the outputs to 0 or 1 at once
//! - Read selected inputs
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

#![deny(unsafe_code)]
#![deny(missing_docs)]
#![no_std]

extern crate embedded_hal as hal;
use hal::blocking::i2c::Write;

/// All possible errors in this crate
#[derive(Debug)]
pub enum Error<E> {
    /// I²C bus error
    I2C(E),
}

/// I/O pin flags, used to select which pins to read in the `get` functions.
/// It is possible to select multiple of them using the _or_ operator.
/// ```
/// # use pcf857x::PinFlags;
/// let pins_to_be_read = PinFlags::P0 | PinFlags::P1;
/// ```
/// Note that P10-17 can only be used with PCF8575 devices.
pub struct PinFlags;

impl PinFlags {
    /// Pin 0
    pub const P0 :  u8 = 0b0000_0001;
    /// Pin 1
    pub const P1 :  u8 = 0b0000_0010;
    /// Pin 2
    pub const P2 :  u8 = 0b0000_0100;
    /// Pin 3
    pub const P3 :  u8 = 0b0000_1000;
    /// Pin 4
    pub const P4 :  u8 = 0b0001_0000;
    /// Pin 5
    pub const P5 :  u8 = 0b0010_0000;
    /// Pin 6
    pub const P6 :  u8 = 0b0100_0000;
    /// Pin 7
    pub const P7 :  u8 = 0b1000_0000;

    /// Pin 10 (only PCF8575)
    pub const P10: u16 = 0b0000_0001_0000_0000;
    /// Pin 11 (only PCF8575)
    pub const P11: u16 = 0b0000_0010_0000_0000;
    /// Pin 12 (only PCF8575)
    pub const P12: u16 = 0b0000_0100_0000_0000;
    /// Pin 13 (only PCF8575)
    pub const P13: u16 = 0b0000_1000_0000_0000;
    /// Pin 14 (only PCF8575)
    pub const P14: u16 = 0b0001_0000_0000_0000;
    /// Pin 15 (only PCF8575)
    pub const P15: u16 = 0b0010_0000_0000_0000;
    /// Pin 16 (only PCF8575)
    pub const P16: u16 = 0b0100_0000_0000_0000;
    /// Pin 17 (only PCF8575)
    pub const P17: u16 = 0b1000_0000_0000_0000;
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

macro_rules! pcf8574 {
    ( $device_name:ident, $default_address:expr ) => {
        /// Device driver
        #[derive(Debug, Default)]
        pub struct $device_name<I2C> {
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
                $device_name {
                    i2c,
                    address: address.addr($default_address),
                    last_set_mask: 0
                }
            }

            /// Destroy driver instance, return I²C bus instance.
            pub fn destroy(self) -> I2C {
                self.i2c
            }

            /// Set the status of all I/O pins.
            pub fn set(&mut self, bits: u8) -> Result<(), Error<E>> {
                self.i2c
                    .write(self.address, &[bits])
                    .map_err(Error::I2C)?;
                self.last_set_mask = bits;
                Ok(())
            }
        }

        impl<I2C, E> $device_name<I2C>
        where
            I2C: hal::blocking::i2c::Read<Error = E> + Write<Error = E>
        {
            /// Get the status of the selected I/O pins.
            /// The mask of the pins to be read can be created with a combination of
            /// `PinFlags::P0` to `PinFlags::P7`.
            pub fn get(&mut self, mask: u8) -> Result<u8, Error<E>> {
                let mask = mask | self.last_set_mask;
                // configure selected pins as inputs
                self.i2c
                    .write(self.address, &[mask])
                    .map_err(Error::I2C)?;

                let mut bits = [0];
                self.i2c
                    .read(self.address, &mut bits)
                    .map_err(Error::I2C).and(Ok(bits[0]))
            }
        }

    };
}

pcf8574!(PCF8574,  0b010_0000);
pcf8574!(PCF8574A, 0b011_1000);


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
}

impl<I2C, E> PCF8575<I2C>
where
    I2C: hal::blocking::i2c::Read<Error = E> + Write<Error = E>
{
    /// Get the status of the selected I/O pins.
    /// The mask of the pins to be read can be created with a combination of
    /// `PinFlags::P0` to `PinFlags::P17`.
    pub fn get(&mut self, mask: u16) -> Result<u16, Error<E>> {
        let mask = mask | self.last_set_mask;
        // configure selected pins as inputs
        self.i2c
            .write(self.address, &u16_to_u8_array(mask)[..])
            .map_err(Error::I2C)?;

        let mut bits = [0; 2];
        self.i2c
            .read(self.address, &mut bits)
            .map_err(Error::I2C).and(Ok(u8_array_to_u16(bits)))
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
    fn can_convert_u16_to_u8_array() {
        assert_eq!([0xCD, 0xAB], u16_to_u8_array(0xABCD));
    }

    #[test]
    fn can_convert_u8_array_to_u16() {
        assert_eq!(0xABCD, u8_array_to_u16([0xCD, 0xAB]));
    }
}

