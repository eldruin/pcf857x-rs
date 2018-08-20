//! This is a platform agnostic Rust driver for the PCF8574, PCF8574A and PCF8575
//! I/O expanders, based on the [`embedded-hal`] traits.
//! [`embedded-hal`]: https://github.com/japaric/embedded-hal

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
/// It is possible to select multiple of them using the _or_ operator:
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


/// PCF8574 driver
#[derive(Debug, Default)]
pub struct PCF8574<I2C> {
    /// The concrete I²C device implementation.
    i2c: I2C,
    /// The I²C device address.
    address: u8,
    /// Last status set to output pins, used to conserve its status while doing a read.
    last_set_mask: u8,
}

impl<I2C, E> PCF8574<I2C>
where
    I2C: Write<Error = E>
{
    /// Create new instance of a PCF8574 device
    pub fn new(i2c: I2C, address: SlaveAddr) -> Self {
        let default_address = 0b010_0000;
        PCF8574 {
            i2c,
            address: address.addr(default_address),
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

impl<I2C, E> PCF8574<I2C>
where
    I2C: hal::blocking::i2c::Read<Error = E> + Write<Error = E>
{
    /// Get the status of the selected I/O pins.
    /// The mask of the pins to be read can be created with a combination of
    /// `PinFlags::P0` to `PinFlags::P7`.
    pub fn get(&mut self, mask: u8) -> Result<u8, Error<E>> {
        read_pins(&mut self.i2c, self.address, mask | self.last_set_mask)
    }
}

/// PCF8574A driver
#[derive(Debug, Default)]
pub struct PCF8574A<I2C> {
    /// The concrete I²C device implementation.
    i2c: I2C,
    /// The I²C device address.
    address: u8,
    /// Last status set to output pins, used to conserve its status while doing a read.
    last_set_mask: u8,
}

impl<I2C, E> PCF8574A<I2C>
where
    I2C: Write<Error = E>
{
    /// Create new instance of a PCF8574 device
    pub fn new(i2c: I2C, address: SlaveAddr) -> Self {
        let default_address = 0b011_1000;
        PCF8574A {
            i2c,
            address: address.addr(default_address),
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

impl<I2C, E> PCF8574A<I2C>
where
    I2C: hal::blocking::i2c::Read<Error = E> + Write<Error = E>
{
    /// Get the status of the selected I/O pins.
    /// The mask of the pins to be read can be created with a combination of
    /// `PinFlags::P0` to `PinFlags::P7`.
    pub fn get(&mut self, mask: u8) -> Result<u8, Error<E>> {
        read_pins(&mut self.i2c, self.address, mask | self.last_set_mask)
    }
}


fn read_pins<I2C, E>(i2c: &mut I2C, address: u8, mask: u8) -> Result<u8, Error<E>>
where
    I2C: hal::blocking::i2c::Read<Error = E> + Write<Error = E>
{
    // configure selected pins as inputs
    i2c.write(address, &[mask])
        .map_err(Error::I2C)?;

    let mut bits = [0];
    i2c.read(address, &mut bits)
       .map_err(Error::I2C).and(Ok(bits[0]))
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
}

