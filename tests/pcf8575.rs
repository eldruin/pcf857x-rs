extern crate pcf857x;
extern crate embedded_hal_mock as hal;
use pcf857x::{PCF8575, SlaveAddr, PinFlags};

fn setup<'a>(data: &'a[u8]) -> PCF8575<hal::I2cMock<'a>> {
    let mut dev = hal::I2cMock::new();
    dev.set_read_data(&data);
    PCF8575::new(dev, SlaveAddr::default())
}

fn check_sent_data(expander: PCF8575<hal::I2cMock>, data: &[u8]) {
    let dev = expander.destroy();
    assert_eq!(dev.get_last_address(), Some(0b010_0000));
    assert_eq!(dev.get_write_data(), &data[..]);
}

fn u16_to_u8_array(input: u16) -> [u8; 2] {
    [input as u8, (input >> 8) as u8]
}

#[test]
fn can_set_output_values() {
    let status = 0b1010_1010_1010_1010;
    let mut expander = setup(&[0]);
    expander.set(status).unwrap();
    check_sent_data(expander, &u16_to_u8_array(status));
}

#[test]
fn can_read_pins() {
    let mut expander = setup(&[0x00, 0x01]);
    let mask = PinFlags::P10 | PinFlags::P17;
    let status = expander.get(mask).unwrap();
    check_sent_data(expander, &u16_to_u8_array(mask));
    assert_eq!(0x0100, status);
}

#[test]
fn read_conserves_output_high_pins() {
    let mut expander = setup(&[0x00, 0x01]);
    let write_status = 0b0101_0101_0101_0101;
    expander.set(write_status).unwrap();
    let mask = PinFlags::P10 | PinFlags::P17;
    let read_status = expander.get(mask).unwrap();
    check_sent_data(expander, &u16_to_u8_array(mask | write_status));
    assert_eq!(0x0100, read_status);
}
