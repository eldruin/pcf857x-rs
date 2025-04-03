use super::super::split_pins;
use super::super::{Error, Pcf8574, Pcf8574a, Pcf8575, PinFlag};
use embedded_hal::i2c::I2c;

macro_rules! pcf8574_set_pin_impl {
    ( $( $device_name:ident ),+ ) => {
        $(
            // The type is PinFlags everywhere and for compatibility
            // with PCF8575. This is only internal so users cannot call this function
            // with the wrong pin number.
            // The methods require only an immutable reference but the actual mutable device
            // is wrapped in a RefCell and will be aquired mutably on execution.
            // Again, this is only internal so users cannot misuse it.
            impl<I2C, E> split_pins::SetPin<E> for $device_name<I2C>
            where
                I2C: I2c<Error = E>,
                E: core::fmt::Debug
            {
                fn set_pin_high(&self, pin_flag: PinFlag) -> Result<(), Error<E>> {
                    self.do_on_acquired(|dev|{
                    let new_mask = dev.last_set_mask | pin_flag.mask as u8;
                    Self::_set(dev, new_mask)
                    })
                }

                fn set_pin_low(&self, pin_flag: PinFlag) -> Result<(), Error<E>> {
                    self.do_on_acquired(|dev|{
                    let new_mask = dev.last_set_mask & !pin_flag.mask as u8;
                    Self::_set(dev, new_mask)
                    })
                }
            }
        )*
    }
}

pcf8574_set_pin_impl!(Pcf8574, Pcf8574a);

impl<I2C, E> split_pins::SetPin<E> for Pcf8575<I2C>
where
    I2C: I2c<Error = E>,
    E: core::fmt::Debug
{
    fn set_pin_high(&self, pin_flag: PinFlag) -> Result<(), Error<E>> {
        self.do_on_acquired(|dev| {
            let new_mask = dev.last_set_mask | pin_flag.mask;
            Self::_set(dev, new_mask)
        })
    }

    fn set_pin_low(&self, pin_flag: PinFlag) -> Result<(), Error<E>> {
        self.do_on_acquired(|dev| {
            let new_mask = dev.last_set_mask & !pin_flag.mask;
            Self::_set(dev, new_mask)
        })
    }
}
