#![deny(unsafe_code)]
#![deny(missing_docs)]

extern crate embedded_hal as hal;
use hal::blocking::i2c::Write;

use super::super::{ Error, PCF8574, PCF8574A, PinFlag };
use super::super::pins;


macro_rules! set_pin_impl {
    ( $( $device_name:ident ),+ ) => {
        $(
            // The type is PinFlags everywhere and for compatibility
            // with PCF8575. This is only internal so users cannot call this function
            // with the wrong pin number.
            // The methods require only an immutable reference but the actual mutable device
            // is wrapped in a RefCell and will be aquired mutably on execution.
            // Again, this is only internal so users cannot misuse it.
            impl<I2C, E> pins::SetPin<E> for $device_name<I2C>
            where
                I2C: Write<Error = E>
            {
                fn set_pin_high(&self, pin_flag: PinFlag) -> Result<(), Error<E>> {
                    let mut dev = self.acquire_device()?;
                    let new_mask = dev.last_set_mask | pin_flag.mask as u8;
                    if dev.last_set_mask != new_mask {
                        let address = dev.address;
                        dev.i2c
                            .write(address, &[new_mask])
                            .map_err(Error::I2C)?;
                        dev.last_set_mask = new_mask;
                    }
                    Ok(())
                }

                fn set_pin_low(&self, pin_flag: PinFlag) -> Result<(), Error<E>> {
                    let mut dev = self.acquire_device()?;
                    let new_mask = dev.last_set_mask & !pin_flag.mask as u8;
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

set_pin_impl!(PCF8574, PCF8574A);