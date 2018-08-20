extern crate pcf857x;
extern crate embedded_hal_mock as hal;
use pcf857x::{PCF8574A, SlaveAddr, PinFlags};

fn setup<'a>(data: &'a[u8]) -> PCF8574A<hal::I2cMock<'a>> {
    let mut dev = hal::I2cMock::new();
    dev.set_read_data(&data);
    PCF8574A::new(dev, SlaveAddr::default())
}

fn check_sent_data(expander: PCF8574A<hal::I2cMock>, data: &[u8]) {
    let dev = expander.destroy();
    assert_eq!(dev.get_last_address(), Some(0b011_1000));
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
fn read_conserves_output_high_pins() {
    let mut expander = setup(&[0x01]);
    let write_status = 0b0101_1010;
    expander.set(write_status).unwrap();
    let mask = PinFlags::P0 | PinFlags::P7;
    let read_status = expander.get(mask).unwrap();
    check_sent_data(expander, &[mask | write_status]);
    assert_eq!(0x01, read_status);
}

