extern crate pcf857x;
extern crate embedded_hal_mock as hal;
use pcf857x::{PCF8574, SlaveAddr, PinFlags};

fn setup<'a>(data: &'a[u8]) -> PCF8574<hal::I2cMock<'a>> {
    let mut dev = hal::I2cMock::new();
    dev.set_read_data(&data);
    PCF8574::new(dev, SlaveAddr::default())
}

fn check_sent_data(expander: PCF8574<hal::I2cMock>, data: &[u8]) {
    let dev = expander.destroy();
    assert_eq!(dev.get_last_address(), Some(0b010_0000));
    assert_eq!(dev.get_write_data(), &data[..]);
}


#[test]
fn can_set_output_values() {
    let status = 0b1010_1010;
    let mut expander = setup(&[0]);
    expander.set(status).unwrap();
    check_sent_data(expander, &[status]);
}

#[test]
fn can_read_pins() {
    let mut expander = setup(&[0x01]);
    let mask = PinFlags::P0 | PinFlags::P7;
    let status = expander.get(mask).unwrap();
    check_sent_data(expander, &[mask]);
    assert_eq!(0x01, status);
}

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
}
