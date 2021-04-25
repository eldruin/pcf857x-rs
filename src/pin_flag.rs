//! Pin flag definition

/// I/O pin flags, used to select which pins to read in the `get` functions.
/// It is possible to select multiple of them using the binary _or_ operator (`|`).
/// ```
/// # use pcf857x::PinFlag;
/// let pins_to_be_read = PinFlag::P0 | PinFlag::P1;
/// ```
/// Note that P10-17 can only be used with PCF8575 devices.
#[derive(Debug, Clone)]
pub struct PinFlag {
    pub(crate) mask: u16,
}

impl PinFlag {
    /// Pin 0
    pub const P0: PinFlag = PinFlag { mask: 1 };
    /// Pin 1
    pub const P1: PinFlag = PinFlag { mask: 2 };
    /// Pin 2
    pub const P2: PinFlag = PinFlag { mask: 4 };
    /// Pin 3
    pub const P3: PinFlag = PinFlag { mask: 8 };
    /// Pin 4
    pub const P4: PinFlag = PinFlag { mask: 16 };
    /// Pin 5
    pub const P5: PinFlag = PinFlag { mask: 32 };
    /// Pin 6
    pub const P6: PinFlag = PinFlag { mask: 64 };
    /// Pin 7
    pub const P7: PinFlag = PinFlag { mask: 128 };

    /// Pin 10 (only PCF8575)
    pub const P10: PinFlag = PinFlag { mask: 256 };
    /// Pin 11 (only PCF8575)
    pub const P11: PinFlag = PinFlag { mask: 512 };
    /// Pin 12 (only PCF8575)
    pub const P12: PinFlag = PinFlag { mask: 1024 };
    /// Pin 13 (only PCF8575)
    pub const P13: PinFlag = PinFlag { mask: 2048 };
    /// Pin 14 (only PCF8575)
    pub const P14: PinFlag = PinFlag { mask: 4096 };
    /// Pin 15 (only PCF8575)
    pub const P15: PinFlag = PinFlag { mask: 8192 };
    /// Pin 16 (only PCF8575)
    pub const P16: PinFlag = PinFlag { mask: 16384 };
    /// Pin 17 (only PCF8575)
    pub const P17: PinFlag = PinFlag { mask: 32768 };
}

use core::ops::BitOr;

impl BitOr for PinFlag {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        PinFlag {
            mask: self.mask | rhs.mask,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PinFlag;

    #[test]
    fn pin_flags_are_correct() {
        assert_eq!(1, PinFlag::P0.mask);
        assert_eq!(2, PinFlag::P1.mask);
        assert_eq!(4, PinFlag::P2.mask);
        assert_eq!(8, PinFlag::P3.mask);
        assert_eq!(16, PinFlag::P4.mask);
        assert_eq!(32, PinFlag::P5.mask);
        assert_eq!(64, PinFlag::P6.mask);
        assert_eq!(128, PinFlag::P7.mask);

        assert_eq!(1 << 8, PinFlag::P10.mask);
        assert_eq!(2 << 8, PinFlag::P11.mask);
        assert_eq!(4 << 8, PinFlag::P12.mask);
        assert_eq!(8 << 8, PinFlag::P13.mask);
        assert_eq!(16 << 8, PinFlag::P14.mask);
        assert_eq!(32 << 8, PinFlag::P15.mask);
        assert_eq!(64 << 8, PinFlag::P16.mask);
        assert_eq!(128 << 8, PinFlag::P17.mask);
    }
}
