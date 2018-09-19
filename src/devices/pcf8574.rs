#![deny(unsafe_code)]
#![deny(missing_docs)]

extern crate embedded_hal as hal;
use hal::blocking::i2c::Write;
pub use hal::digital::OutputPin;

#[cfg(feature = "std")]
use std::cell;

#[cfg(not(feature = "std"))]
use core::cell;

use super::super::pins::pcf8574;
use super::super::{ SlaveAddr, Error, PinFlag };


macro_rules! pcf8574 {
    ( $device_name:ident, $device_data_name:ident, $default_address:expr ) => {
        /// Device driver
        #[derive(Debug, Default)]
        pub struct $device_name<I2C> {
            /// Data
            pub(crate) data: cell::RefCell<$device_data_name<I2C>>
        }

        #[derive(Debug, Default)]
        pub(crate) struct $device_data_name<I2C> {
            /// The concrete I²C device implementation.
            pub(crate) i2c: I2C,
            /// The I²C device address.
            pub(crate) address: u8,
            /// Last status set to output pins, used to conserve its status while doing a read.
            pub(crate) last_set_mask: u8,
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

            pub(crate) fn acquire_device(&self) -> Result<cell::RefMut<$device_data_name<I2C>>, Error<E>> {
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
