use core::cell;
use embedded_hal::blocking::i2c::{Read, Write};
pub use embedded_hal::digital::v2::OutputPin;

use crate::pins::pcf8575;
use crate::{Error, PinFlag, SlaveAddr};

/// PCF8575 device driver
#[derive(Debug, Default)]
pub struct Pcf8575<I2C> {
    /// Device
    dev: cell::RefCell<Pcf8575Data<I2C>>,
}

#[derive(Debug, Default)]
pub(crate) struct Pcf8575Data<I2C> {
    /// The concrete I²C device implementation.
    pub(crate) i2c: I2C,
    /// The I²C device address.
    pub(crate) address: u8,
    /// Last status set to output pins, used to conserve its status while doing a read.
    pub(crate) last_set_mask: u16,
}

impl<I2C, E> Pcf8575<I2C>
where
    I2C: Write<Error = E>,
{
    /// Create new instance of the PCF8575 device
    pub fn new(i2c: I2C, address: SlaveAddr) -> Self {
        let dev = Pcf8575Data {
            i2c,
            address: address.addr(0b010_0000),
            last_set_mask: 0,
        };
        Pcf8575 {
            dev: cell::RefCell::new(dev),
        }
    }

    /// Destroy driver instance, return I²C bus instance.
    pub fn destroy(self) -> I2C {
        self.dev.into_inner().i2c
    }

    /// Set the status of all I/O pins.
    pub fn set(&mut self, bits: u16) -> Result<(), Error<E>> {
        self.do_on_acquired(|dev| Self::_set(dev, bits))
    }

    pub(crate) fn _set(mut dev: cell::RefMut<Pcf8575Data<I2C>>, bits: u16) -> Result<(), Error<E>> {
        let address = dev.address;
        dev.i2c
            .write(address, &u16_to_u8_array(bits)[..])
            .map_err(Error::I2C)?;
        dev.last_set_mask = bits;
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
            self.do_on_acquired(|mut dev| {
                let address = dev.address;
                dev.i2c.write(address, &data).map_err(Error::I2C)?;
                dev.last_set_mask =
                    (u16::from(data[data.len() - 1]) << 8) | u16::from(data[data.len() - 2]);
                Ok(())
            })?;
        }
        Ok(())
    }

    /// Split device into individual pins
    pub fn split<'a>(&'a self) -> pcf8575::Parts<'a, Pcf8575<I2C>, E> {
        pcf8575::Parts::new(&self)
    }

    pub(crate) fn do_on_acquired<R>(
        &self,
        f: impl FnOnce(cell::RefMut<Pcf8575Data<I2C>>) -> Result<R, Error<E>>,
    ) -> Result<R, Error<E>> {
        let dev = self
            .dev
            .try_borrow_mut()
            .map_err(|_| Error::CouldNotAcquireDevice)?;
        f(dev)
    }
}

impl<I2C, E> Pcf8575<I2C>
where
    I2C: Read<Error = E> + Write<Error = E>,
{
    /// Get the status of the selected I/O pins.
    /// The mask of the pins to be read can be created with a combination of
    /// `PinFlag::P0` to `PinFlag::P17`.
    pub fn get(&mut self, mask: &PinFlag) -> Result<u16, Error<E>> {
        self.do_on_acquired(|dev| Self::_get(dev, mask))
    }

    pub(crate) fn _get(
        mut dev: cell::RefMut<Pcf8575Data<I2C>>,
        mask: &PinFlag,
    ) -> Result<u16, Error<E>> {
        let address = dev.address;
        let mask = mask.mask | dev.last_set_mask;
        // configure selected pins as inputs
        dev.i2c
            .write(address, &u16_to_u8_array(mask)[..])
            .map_err(Error::I2C)?;

        let mut bits = [0; 2];
        dev.i2c
            .read(address, &mut bits)
            .map_err(Error::I2C)
            .and(Ok(u8_array_to_u16(bits)))
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
            self.do_on_acquired(|mut dev| {
                let address = dev.address;
                let mask = mask.mask | dev.last_set_mask;
                // configure selected pins as inputs
                dev.i2c
                    .write(address, &u16_to_u8_array(mask))
                    .map_err(Error::I2C)?;

                dev.i2c.read(address, &mut data).map_err(Error::I2C)
            })?;
        }
        Ok(())
    }
}

fn u16_to_u8_array(input: u16) -> [u8; 2] {
    [input as u8, (input >> 8) as u8]
}

fn u8_array_to_u16(input: [u8; 2]) -> u16 {
    u16::from(input[0]) | (u16::from(input[1]) << 8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_convert_u16_to_u8_array() {
        assert_eq!([0xCD, 0xAB], u16_to_u8_array(0xABCD));
    }

    #[test]
    fn can_convert_u8_array_to_u16() {
        assert_eq!(0xABCD, u8_array_to_u16([0xCD, 0xAB]));
    }
}
