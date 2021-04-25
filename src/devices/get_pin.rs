use crate::split_pins;
use crate::{Error, Pcf8574, Pcf8574a, Pcf8575, PinFlag};
use embedded_hal::blocking::i2c::{Read, Write};

macro_rules! pcf8574_get_pin_impl {
    ( $( $device_name:ident ),+ ) => {
        $(
            // The type is PinFlags everywhere and for compatibility
            // with PCF8575. This is only internal so users cannot call this function
            // with the wrong pin number.
            // The methods require only an immutable reference but the actual mutable device
            // is wrapped in a RefCell and will be aquired mutably on execution.
            // Again, this is only internal so users cannot misuse it.
            impl<I2C, E> split_pins::GetPin<E> for $device_name<I2C>
            where
                I2C: Read<Error = E> + Write<Error = E>
            {
                fn is_pin_high(&self, pin_flag: PinFlag) -> Result<bool, Error<E>> {
                    self.do_on_acquired(|dev|{
                    let data = Self::_get(dev, pin_flag)?;
                    Ok(data & pin_flag.mask as u8 != 0)
                    })
                }

                fn is_pin_low(&self, pin_flag: PinFlag) -> Result<bool, Error<E>> {
                    self.do_on_acquired(|dev|{
                    let data = Self::_get(dev, pin_flag)?;
                    Ok(data & pin_flag.mask as u8 == 0)
                    })
                }
            }
        )*
    }
}

pcf8574_get_pin_impl!(Pcf8574, Pcf8574a);

impl<I2C, E> split_pins::GetPin<E> for Pcf8575<I2C>
where
    I2C: Read<Error = E> + Write<Error = E>,
{
    fn is_pin_high(&self, pin_flag: PinFlag) -> Result<bool, Error<E>> {
        self.do_on_acquired(|dev| {
            let data = Self::_get(dev, pin_flag)?;
            Ok(data & pin_flag.mask != 0)
        })
    }

    fn is_pin_low(&self, pin_flag: PinFlag) -> Result<bool, Error<E>> {
        self.do_on_acquired(|dev| {
            let data = Self::_get(dev, pin_flag)?;
            Ok(data & pin_flag.mask == 0)
        })
    }
}
