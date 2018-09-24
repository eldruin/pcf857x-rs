extern crate pcf857x;
extern crate embedded_hal_mock as hal;
use pcf857x::{PCF8575, SlaveAddr, Error, PinFlag};

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

fn check_nothing_was_sent(expander: PCF8575<hal::I2cMock>) {
    let dev = expander.destroy();
    assert!(dev.get_last_address().is_none());
    assert!(dev.get_write_data().is_empty());
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
fn can_write_multiple_words() {
    let data = [0b1010_1010, 0b0101_0101];
    let mut expander = setup(&[0]);
    expander.write_array(&data).unwrap();
    check_sent_data(expander, &data);
}

#[test]
fn write_empty_array_does_nothing() {
    let mut expander = setup(&[0]);
    expander.write_array(&[]).unwrap();
    check_nothing_was_sent(expander);
}

#[test]
fn write_array_with_odd_word_count_returns_error() {
    let mut expander = setup(&[0]);
    match expander.write_array(&[0]) {
        Err(Error::InvalidInputData) => (),
        _ => panic!()
    }
}

 #[test]
fn read_multiple_words_with_odd_size_array_returns_error() {
    let mut data = [0; 3];
    let mut expander = setup(&[0xAB, 0xCD]);
    let mask = PinFlag::P0 | PinFlag::P17;
    match expander.read_array(&mask, &mut data) {
        Err(Error::InvalidInputData) => (),
        _ => panic!()
    }
}

macro_rules! pin_test {
    ($px:ident, $value:expr) => {
        mod $px {
            use super::*;
            use pcf857x::OutputPin;
            #[cfg(feature = "unproven")]
            use pcf857x::InputPin;

            #[test]
            fn can_split_and_set_high() {
                let expander = setup(&[0]);
                {
                  let mut parts = expander.split();
                  parts.$px.set_high();
                }
                check_sent_data(expander, &u16_to_u8_array($value)[..]);
            }

            #[test]
            fn can_split_and_set_low() {
                let mut expander = setup(&[0]);
                expander.set(0b1111_1111_1111_1111).unwrap();
                {
                  let mut parts = expander.split();
                  parts.$px.set_low();
                }
                let data = 0b1111_1111_1111_1111 & !$value;
                check_sent_data(expander, &u16_to_u8_array(data)[..]);
            }

            #[cfg(feature = "unproven")]
            #[test]
            fn can_split_and_get_is_high() {
                let input = u16_to_u8_array($value);
                let expander = setup(&input);
                {
                  let parts = expander.split();
                  assert!(parts.$px.is_high());
                }
                check_sent_data(expander, &u16_to_u8_array($value)[..]);
            }

            #[cfg(feature = "unproven")]
            #[test]
            fn can_split_and_get_is_low() {
                let input = u16_to_u8_array(!$value);
                let expander = setup(&input);
                {
                  let parts = expander.split();
                  assert!(parts.$px.is_low());
                }
                check_sent_data(expander, &u16_to_u8_array($value)[..]);
            }
        }
    }
}

pin_test!(p0,      1);
pin_test!(p1,      2);
pin_test!(p2,      4);
pin_test!(p3,      8);
pin_test!(p4,     16);
pin_test!(p5,     32);
pin_test!(p6,     64);
pin_test!(p7,    128);
pin_test!(p10,   256);
pin_test!(p11,   512);
pin_test!(p12,  1024);
pin_test!(p13,  2048);
pin_test!(p14,  4096);
pin_test!(p15,  8192);
pin_test!(p16, 16384);
pin_test!(p17, 32768);
