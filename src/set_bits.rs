#![deny(unsafe_code)]
#![deny(missing_docs)]

extern crate embedded_hal as hal;
use hal::blocking::i2c::Write;

use super::{ Error, PCF8574, PCF8574A};

/// Set a number of bits high or low
pub trait SetBits<T, E> {
    /// Set a number of bits high
    fn set_bits_high(&self, bitmask: T) -> Result<(), Error<E>>;
    /// Set a number of bits low
    fn set_bits_low (&self, bitmask: T) -> Result<(), Error<E>>;
}

macro_rules! set_bits_impl {
    ( $( $device_name:ident ),+ ) => {
        $(
            // The type is u16 here to reuse PinFlags everywhere and for compatibility
            // with PCF8575. This is only internal so users cannot misuse it.
            // The methods require only an immutable reference but the actual mutable device
            // is wrapped in a RefCell and will be aquired mutably on execution.
            // Again, this is only internal so users cannot misuse it.
            impl<I2C, E> SetBits<u16, E> for $device_name<I2C>
            where
                I2C: Write<Error = E>
            {
                fn set_bits_high(&self, bitmask: u16) -> Result<(), Error<E>> {
                    let mut dev = self.acquire_device()?;
                    let new_mask = dev.last_set_mask | bitmask as u8;
                    if dev.last_set_mask != new_mask {
                        let address = dev.address;
                        dev.i2c
                            .write(address, &[new_mask])
                            .map_err(Error::I2C)?;
                        dev.last_set_mask = new_mask;
                    }
                    Ok(())
                }

                fn set_bits_low(&self, bitmask: u16) -> Result<(), Error<E>> {
                    let mut dev = self.acquire_device()?;
                    let new_mask = dev.last_set_mask & !bitmask as u8;
                    if dev.last_set_mask != new_mask {
                        let address = dev.address;
                        dev.i2c
                            .write(address, &[new_mask])
                            .map_err(Error::I2C)?;
                        dev.last_set_mask = new_mask;
                    }
                    Ok(())
                }
            }
        )*
    }
}

set_bits_impl!(PCF8574, PCF8574A);
