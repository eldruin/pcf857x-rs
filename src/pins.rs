#![deny(unsafe_code)]
#![deny(missing_docs)]

extern crate embedded_hal as hal;
pub use hal::digital::OutputPin;

#[cfg(feature = "unproven")]
pub use hal::digital::InputPin;

use super::{ Error, PinFlag };

#[cfg(feature = "std")]
use std::marker;

#[cfg(not(feature = "std"))]
use core::marker;

use self::marker::PhantomData;

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

/// Module containing structures specific to PCF8575
pub mod pcf8575 {
    parts!(p0,  P0,  p1,  P1,  p2,  P2,  p3,  P3,  p4,  P4,  p5,  P5,  p6,  P6,  p7,  P7,
           p10, P10, p11, P11, p12, P12, p13, P13, p14, P14, p15, P15, p16, P16, p17, P17);
}

/// Set a pin high or low
pub trait SetPin<E> {
    /// Set a pin high
    fn set_pin_high(&self, pin_flag: PinFlag) -> Result<(), Error<E>>;
    /// Set a pin low
    fn set_pin_low (&self, pin_flag: PinFlag) -> Result<(), Error<E>>;
}

/// Read if a pin is high or low
pub trait GetPin<E> {
    /// Reads a pin and returns whether it is high
    fn is_pin_high(&self, pin_flag: PinFlag) -> Result<bool, Error<E>>;
    /// Reads a pin and returns whether it is low
    fn is_pin_low (&self, pin_flag: PinFlag) -> Result<bool, Error<E>>;
}

macro_rules! io_pin_impl {
    ( $( $PX:ident ),+ ) => {
        $(
            impl<'a, S, E> OutputPin for $PX<'a, S, E>
            where S: SetPin<E> {

                fn set_high(&mut self) {
                    match self.0.set_pin_high(PinFlag::$PX) {
                        Err(Error::CouldNotAcquireDevice) => panic!("Could not set pin to high. Could not acquire device."),
                        Err(_) => panic!("Could not set pin to high."),
                        _ => ()
                    }
                }

                fn set_low(&mut self) {
                    match self.0.set_pin_low(PinFlag::$PX) {
                        Err(Error::CouldNotAcquireDevice) => panic!("Could not set pin to high. Could not acquire device."),
                        Err(_) => panic!("Could not set pin to high."),
                        _ => ()
                    }
                }
            }

            #[cfg(feature = "unproven")]
            impl<'a, S, E> InputPin for $PX<'a, S, E>
            where S: GetPin<E> {

                fn is_high(&self) -> bool {
                    match self.0.is_pin_high(PinFlag::$PX) {
                        Err(Error::CouldNotAcquireDevice) => panic!("Could not read pin status. Could not acquire device."),
                        Err(_) => panic!("Could not read pin status."),
                        Ok(value) => value
                    }
                }

                fn is_low(&self) -> bool {
                    match self.0.is_pin_low(PinFlag::$PX) {
                        Err(Error::CouldNotAcquireDevice) => panic!("Could not read pin status. Could not acquire device."),
                        Err(_) => panic!("Could not read pin status."),
                        Ok(value) => value
                    }
                }
            }
        )*
    }
}

io_pin_impl!( P0,  P1,  P2,  P3,  P4,  P5,  P6,  P7,
             P10, P11, P12, P13, P14, P15, P16, P17 );


