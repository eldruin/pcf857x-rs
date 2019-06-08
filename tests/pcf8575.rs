extern crate embedded_hal_mock as hal;
use hal::i2c::{Mock as I2cMock, Transaction as I2cTrans};
extern crate pcf857x;
use pcf857x::{Error, Pcf8575, PinFlag, SlaveAddr};
mod base;

const DEV_ADDR: u8 = 0b010_0000;

pub fn new(transactions: &[I2cTrans]) -> Pcf8575<I2cMock> {
    Pcf8575::new(I2cMock::new(&transactions), SlaveAddr::default())
}

fn u16_to_u8_array(input: u16) -> [u8; 2] {
    [input as u8, (input >> 8) as u8]
}

#[test]
fn can_set_output_values() {
    let status = 0b1010_1010_1010_1010;
    let transactions = [I2cTrans::write(DEV_ADDR, u16_to_u8_array(status).to_vec())];
    let mut expander = new(&transactions);
    expander.set(status).unwrap();
    expander.destroy().done();
}

#[test]
fn can_write_multiple_words() {
    let data = [0b0101_0101, 0b1010_1010];
    let transactions = [I2cTrans::write(DEV_ADDR, data.to_vec())];
    let mut expander = new(&transactions);
    expander.write_array(&data).unwrap();
    expander.destroy().done();
}

#[test]
fn write_empty_array_does_nothing() {
    let mut expander = new(&[]);
    expander.write_array(&[]).unwrap();
    expander.destroy().done();
}

#[test]
fn write_array_with_odd_word_count_returns_error() {
    let mut expander = new(&[]);
    expect_err!(expander.write_array(&[0]), InvalidInputData);
    expander.destroy().done();
}

#[test]
fn read_multiple_words_with_odd_size_array_returns_error() {
    let mut data = [0; 3];
    let mut expander = new(&[]);

    let mask = PinFlag::P0 | PinFlag::P17;
    expect_err!(expander.read_array(&mask, &mut data), InvalidInputData);
    expander.destroy().done();
}

#[test]
fn can_read_pins() {
    let transactions = [
        I2cTrans::write(DEV_ADDR, vec![0x01, 0x80]),
        I2cTrans::read(DEV_ADDR, vec![0x00, 0x80]),
    ];
    let mut expander = new(&transactions);
    let mask = PinFlag::P0 | PinFlag::P17;
    let status = expander.get(&mask).unwrap();
    assert_eq!(0x8000, status);
    expander.destroy().done();
}

#[test]
fn read_conserves_output_high_pins() {
    let write_status = 0b0101_0101_0101_0101;
    let transactions = [
        I2cTrans::write(DEV_ADDR, u16_to_u8_array(write_status).to_vec()),
        I2cTrans::write(
            DEV_ADDR,
            u16_to_u8_array(write_status | 0x01 | 0x8000).to_vec(),
        ),
        I2cTrans::read(DEV_ADDR, vec![0x00, 0x80]),
    ];
    let mut expander = new(&transactions);
    expander.set(write_status).unwrap();
    let mask = PinFlag::P0 | PinFlag::P17;
    let status = expander.get(&mask).unwrap();
    assert_eq!(0x8000, status);
    expander.destroy().done();
}

#[test]
fn can_read_multiple_words() {
    let transactions = [
        I2cTrans::write(DEV_ADDR, vec![0x01, 0x80]),
        I2cTrans::read(DEV_ADDR, vec![0xAB, 0xCD]),
    ];
    let mut expander = new(&transactions);
    let mask = PinFlag::P0 | PinFlag::P17;
    let mut data = [0; 2];
    expander.read_array(&mask, &mut data).unwrap();
    assert_eq!([0xAB, 0xCD], data);
    expander.destroy().done();
}

#[test]
fn reading_multiple_words_conserves_high_pins() {
    let write_status = 0b0101_0101_0101_0101;
    let transactions = [
        I2cTrans::write(DEV_ADDR, u16_to_u8_array(write_status).to_vec()),
        I2cTrans::write(
            DEV_ADDR,
            u16_to_u8_array(write_status | 0x01 | 0x8000).to_vec(),
        ),
        I2cTrans::read(DEV_ADDR, vec![0xAB, 0xCD]),
    ];
    let mut expander = new(&transactions);
    expander.set(write_status).unwrap();
    let mask = PinFlag::P0 | PinFlag::P17;
    let mut data = [0; 2];
    expander.read_array(&mask, &mut data).unwrap();
    assert_eq!([0xAB, 0xCD], data);
    expander.destroy().done();
}

macro_rules! pin_test {
    ($px:ident, $value:expr) => {
        mod $px {
            use super::*;
            #[cfg(feature = "unproven")]
            use pcf857x::InputPin;
            use pcf857x::OutputPin;

            #[test]
            fn can_split_and_set_high() {
                let transactions = [I2cTrans::write(DEV_ADDR, u16_to_u8_array($value).to_vec())];
                let expander = new(&transactions);

                {
                    let mut parts = expander.split();
                    parts.$px.set_high().unwrap();
                }
                expander.destroy().done();
            }

            #[test]
            fn can_split_and_set_low() {
                let transactions = [
                    I2cTrans::write(DEV_ADDR, vec![0b1111_1111, 0b1111_1111]),
                    I2cTrans::write(
                        DEV_ADDR,
                        u16_to_u8_array(0b1111_1111_1111_1111 & !$value).to_vec(),
                    ),
                ];
                let mut expander = new(&transactions);
                expander.set(0b1111_1111_1111_1111).unwrap();
                {
                    let mut parts = expander.split();
                    parts.$px.set_low().unwrap();
                }
                expander.destroy().done();
            }

            #[cfg(feature = "unproven")]
            #[test]
            fn can_split_and_get_is_high() {
                let transactions = [
                    I2cTrans::write(DEV_ADDR, u16_to_u8_array($value).to_vec()),
                    I2cTrans::read(DEV_ADDR, u16_to_u8_array($value).to_vec()),
                ];
                let expander = new(&transactions);

                {
                    let parts = expander.split();
                    assert!(parts.$px.is_high().unwrap());
                }
                expander.destroy().done();
            }

            #[cfg(feature = "unproven")]
            #[test]
            fn can_split_and_get_is_low() {
                let transactions = [
                    I2cTrans::write(DEV_ADDR, u16_to_u8_array($value).to_vec()),
                    I2cTrans::read(DEV_ADDR, u16_to_u8_array(!$value).to_vec()),
                ];
                let expander = new(&transactions);
                {
                    let parts = expander.split();
                    assert!(parts.$px.is_low().unwrap());
                }
                expander.destroy().done();
            }
        }
    };
}

pin_test!(p0, 1);
pin_test!(p1, 2);
pin_test!(p2, 4);
pin_test!(p3, 8);
pin_test!(p4, 16);
pin_test!(p5, 32);
pin_test!(p6, 64);
pin_test!(p7, 128);
pin_test!(p10, 256);
pin_test!(p11, 512);
pin_test!(p12, 1024);
pin_test!(p13, 2048);
pin_test!(p14, 4096);
pin_test!(p15, 8192);
pin_test!(p16, 16384);
pin_test!(p17, 32768);
