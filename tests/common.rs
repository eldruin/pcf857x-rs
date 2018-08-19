extern crate pcf857x;
use pcf857x::PinFlags;

#[test]
fn pin_flags_are_correct() {
    assert_eq!(1,   PinFlags::P0);
    assert_eq!(2,   PinFlags::P1);
    assert_eq!(4,   PinFlags::P2);
    assert_eq!(8,   PinFlags::P3);
    assert_eq!(16,  PinFlags::P4);
    assert_eq!(32,  PinFlags::P5);
    assert_eq!(64,  PinFlags::P6);
    assert_eq!(128, PinFlags::P7);

    assert_eq!(1 << 8,   PinFlags::P10);
    assert_eq!(2 << 8,   PinFlags::P11);
    assert_eq!(4 << 8,   PinFlags::P12);
    assert_eq!(8 << 8,   PinFlags::P13);
    assert_eq!(16 << 8,  PinFlags::P14);
    assert_eq!(32 << 8,  PinFlags::P15);
    assert_eq!(64 << 8,  PinFlags::P16);
    assert_eq!(128 << 8, PinFlags::P17);
}
