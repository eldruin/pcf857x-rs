pub use hal::digital::v2::OutputPin;

#[cfg(feature = "unproven")]
pub use hal::digital::v2::InputPin;

use super::{Error, PinFlag};
use core::marker::PhantomData;

macro_rules! pins {
    ( $( $PX:ident ),+ ) => {
        $(  /// Pin
            pub struct $PX<'a, IC: 'a, E>(&'a IC, PhantomData<E>);
        )*
    }
}
pins!(P0, P1, P2, P3, P4, P5, P6, P7, P10, P11, P12, P13, P14, P15, P16, P17);

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
    parts!(
        p0, P0, p1, P1, p2, P2, p3, P3, p4, P4, p5, P5, p6, P6, p7, P7, p10, P10, p11, P11, p12,
        P12, p13, P13, p14, P14, p15, P15, p16, P16, p17, P17
    );
}

/// Set a pin high or low
pub trait SetPin<E> {
    /// Set a pin high
    fn set_pin_high(&self, pin_flag: PinFlag) -> Result<(), Error<E>>;
    /// Set a pin low
    fn set_pin_low(&self, pin_flag: PinFlag) -> Result<(), Error<E>>;
}

/// Read if a pin is high or low
pub trait GetPin<E> {
    /// Reads a pin and returns whether it is high
    fn is_pin_high(&self, pin_flag: PinFlag) -> Result<bool, Error<E>>;
    /// Reads a pin and returns whether it is low
    fn is_pin_low(&self, pin_flag: PinFlag) -> Result<bool, Error<E>>;
}

macro_rules! io_pin_impl {
    ( $( $PX:ident ),+ ) => {
        $(
            impl<'a, S, E> OutputPin for $PX<'a, S, E>
            where S: SetPin<E> {
                type Error = Error<E>;

                fn set_high(&mut self) -> Result<(), Self::Error> {
                    self.0.set_pin_high(PinFlag::$PX)
                }

                fn set_low(&mut self) -> Result<(), Self::Error> {
                    self.0.set_pin_low(PinFlag::$PX)
                }
            }

            #[cfg(feature = "unproven")]
            impl<'a, S, E> InputPin for $PX<'a, S, E>
            where S: GetPin<E> {
                type Error = Error<E>;

                fn is_high(&self) -> Result<bool, Self::Error> {
                    self.0.is_pin_high(PinFlag::$PX)
                }

                fn is_low(&self) -> Result<bool, Self::Error> {
                    self.0.is_pin_low(PinFlag::$PX)
                }
            }
        )*
    }
}

io_pin_impl!(P0, P1, P2, P3, P4, P5, P6, P7, P10, P11, P12, P13, P14, P15, P16, P17);
