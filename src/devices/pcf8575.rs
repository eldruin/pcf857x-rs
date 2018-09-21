#![deny(unsafe_code)]
#![deny(missing_docs)]

extern crate embedded_hal as hal;
use hal::blocking::i2c::Write;
pub use hal::digital::OutputPin;

#[cfg(feature = "std")]
use std::cell;

#[cfg(not(feature = "std"))]
use core::cell;

use super::super::pins::pcf8575;
use super::super::{ SlaveAddr, Error, PinFlag };

/// PCF8575 device driver
#[derive(Debug, Default)]
pub struct PCF8575<I2C> {
    /// Device
    dev: cell::RefCell<PCF8575Data<I2C>>,
}

#[derive(Debug, Default)]
pub(crate) struct PCF8575Data<I2C> {
    /// The concrete I²C device implementation.
    pub(crate) i2c: I2C,
    /// The I²C device address.
    pub(crate) address: u8,
    /// Last status set to output pins, used to conserve its status while doing a read.
    pub(crate) last_set_mask: u16,
}

impl<I2C, E> PCF8575<I2C>
where
    I2C: Write<Error = E>
{
    /// Create new instance of the PCF8575 device
    pub fn new(i2c: I2C, address: SlaveAddr) -> Self {
        let dev = PCF8575Data {
            i2c,
            address: address.addr(0b010_0000),
            last_set_mask: 0
        };
        PCF8575 {
            dev: cell::RefCell::new(dev)
        }
    }

    /// Destroy driver instance, return I²C bus instance.
    pub fn destroy(self) -> I2C {
        self.dev.into_inner().i2c
    }

    /// Set the status of all I/O pins.
    pub fn set(&mut self, bits: u16) -> Result<(), Error<E>> {
        let dev = self.acquire_device()?;
        Self::_set(dev, bits)
    }

    pub(crate) fn _set(mut dev: cell::RefMut<PCF8575Data<I2C>>, bits: u16) -> Result<(), Error<E>> {
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
            let mut dev = self.acquire_device()?;
            let address = dev.address;
            dev.i2c
                .write(address, &data)
                .map_err(Error::I2C)?;
            dev.last_set_mask = ((data[data.len()-1] as u16) << 8) | data[data.len()-2] as u16;
        }
        Ok(())
    }

    /// Split device into individual pins
    pub fn split<'a>(&'a self) -> pcf8575::Parts<'a, PCF8575<I2C>, E> {
        pcf8575::Parts::new(&self)
    }

    pub(crate) fn acquire_device(&self) -> Result<cell::RefMut<PCF8575Data<I2C>>, Error<E>> {
        self.dev.try_borrow_mut().map_err(|_| Error::CouldNotAcquireDevice)
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
        let dev = self.acquire_device()?;
        Self::_get(dev, mask)
    }

    pub(crate) fn _get(mut dev: cell::RefMut<PCF8575Data<I2C>>, mask: &PinFlag) -> Result<u16, Error<E>> {
        let address = dev.address;
        let mask = mask.mask | dev.last_set_mask;
        // configure selected pins as inputs
        dev.i2c
            .write(address, &u16_to_u8_array(mask)[..])
            .map_err(Error::I2C)?;

        let mut bits = [0; 2];
        dev.i2c
            .read(address, &mut bits)
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
            let mut dev = self.acquire_device()?;
            let address = dev.address;
            let mask = mask.mask | dev.last_set_mask;
            // configure selected pins as inputs
            dev.i2c
                .write(address, &u16_to_u8_array(mask))
                .map_err(Error::I2C)?;

            dev.i2c
                .read(address, &mut data)
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
    fn can_convert_u16_to_u8_array() {
        assert_eq!([0xCD, 0xAB], u16_to_u8_array(0xABCD));
    }

    #[test]
    fn can_convert_u8_array_to_u16() {
        assert_eq!(0xABCD, u8_array_to_u16([0xCD, 0xAB]));
    }

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

