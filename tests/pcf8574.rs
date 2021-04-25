extern crate embedded_hal_mock as hal;
use hal::i2c::{Mock as I2cMock, Transaction as I2cTrans};
extern crate pcf857x;
use pcf857x::{Error, Pcf8574, Pcf8574a, PinFlag, SlaveAddr};
mod base;

macro_rules! pcf8574_tests {
    ($device_name:ident, $test_mod_name:ident, $default_address:expr) => {
        mod $test_mod_name {
            use super::*;

            pub fn new(transactions: &[I2cTrans]) -> $device_name<I2cMock> {
                $device_name::new(I2cMock::new(transactions), SlaveAddr::default())
            }

            #[test]
            fn can_read_pins() {
                let transactions = [
                    I2cTrans::write($default_address, vec![1 | 128]),
                    I2cTrans::read($default_address, vec![0x01]),
                ];
                let mut expander = new(&transactions);
                let mask = PinFlag::P0 | PinFlag::P7;
                let status = expander.get(&mask).unwrap();
                assert_eq!(0x01, status);
                expander.destroy().done();
            }

            #[test]
            fn read_conserves_output_high_pins() {
                let write_status = 0b0101_1010;
                let transactions = [
                    I2cTrans::write($default_address, vec![write_status]),
                    I2cTrans::write($default_address, vec![1 | 128 | write_status]),
                    I2cTrans::read($default_address, vec![0x01]),
                ];
                let mut expander = new(&transactions);
                expander.set(write_status).unwrap();
                let mask = PinFlag::P0 | PinFlag::P7;
                let status = expander.get(&mask).unwrap();
                assert_eq!(0x01, status);
                expander.destroy().done();
            }

            #[test]
            fn can_read_multiple_words() {
                let transactions = [
                    I2cTrans::write($default_address, vec![1 | 128]),
                    I2cTrans::read($default_address, vec![0xAB, 0xCD]),
                ];
                let mut expander = new(&transactions);
                let mut data = [0; 2];
                let mask = PinFlag::P0 | PinFlag::P7;
                expander.read_array(&mask, &mut data).unwrap();
                assert_eq!([0xAB, 0xCD], data);
                expander.destroy().done();
            }

            #[test]
            fn reading_multiple_words_conserves_high_pins() {
                let write_status = 0b0101_1010;
                let transactions = [
                    I2cTrans::write($default_address, vec![write_status]),
                    I2cTrans::write($default_address, vec![1 | 128 | write_status]),
                    I2cTrans::read($default_address, vec![0xAB, 0xCD]),
                ];
                let mut expander = new(&transactions);
                expander.set(write_status).unwrap();
                let mut data = [0; 2];
                let mask = PinFlag::P0 | PinFlag::P7;
                expander.read_array(&mask, &mut data).unwrap();
                assert_eq!([0xAB, 0xCD], data);
                expander.destroy().done();
            }

            #[test]
            fn can_set_output_values() {
                let status = 0b1010_1010;
                let transactions = [I2cTrans::write($default_address, vec![status])];
                let mut expander = new(&transactions);
                expander.set(status).unwrap();
                expander.destroy().done();
            }

            #[test]
            fn read_wrong_pin_flag_returns_error() {
                let mut expander = new(&[]);
                let mask = PinFlag::P0 | PinFlag::P17;
                expect_err!(expander.get(&mask), InvalidInputData);
                expander.destroy().done();
            }

            #[test]
            fn can_write_multiple_words() {
                let data = [0b1010_1010, 0b0101_0101];
                let transactions = [I2cTrans::write($default_address, data.to_vec())];
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
            fn empty_array_read_does_nothing() {
                let mut expander = new(&[]);
                let mask = PinFlag::P0 | PinFlag::P7;
                expander.read_array(&mask, &mut []).unwrap();
                expander.destroy().done();
            }

            #[test]
            fn read_multiple_words_but_wrong_pin_flag_returns_error() {
                let mut data = [0; 2];
                let mut expander = new(&[]);
                let mask = PinFlag::P0 | PinFlag::P17;
                expect_err!(expander.read_array(&mask, &mut data), InvalidInputData);
                expander.destroy().done();
            }
            pcf8574_pin_test!(p0, 1, $default_address);
            pcf8574_pin_test!(p1, 2, $default_address);
            pcf8574_pin_test!(p2, 4, $default_address);
            pcf8574_pin_test!(p3, 8, $default_address);
            pcf8574_pin_test!(p4, 16, $default_address);
            pcf8574_pin_test!(p5, 32, $default_address);
            pcf8574_pin_test!(p6, 64, $default_address);
            pcf8574_pin_test!(p7, 128, $default_address);
        }
    };
}

macro_rules! pcf8574_pin_test {
    ($px:ident, $value:expr, $default_address:expr) => {
        mod $px {
            use super::*;
            #[cfg(feature = "unproven")]
            use pcf857x::InputPin;
            use pcf857x::OutputPin;

            #[test]
            fn can_split_and_set_high() {
                let transactions = [I2cTrans::write($default_address, vec![$value])];
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
                    I2cTrans::write($default_address, vec![0b1111_1111]),
                    I2cTrans::write($default_address, vec![0b1111_1111 & !$value]),
                ];
                let mut expander = new(&transactions);
                expander.set(0b1111_1111).unwrap();
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
                    I2cTrans::write($default_address, vec![$value]),
                    I2cTrans::read($default_address, vec![$value]),
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
                    I2cTrans::write($default_address, vec![$value]),
                    I2cTrans::read($default_address, vec![!$value]),
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

pcf8574_tests!(Pcf8574, pcf8574_tests, 0b010_0000);
pcf8574_tests!(Pcf8574a, pcf8574a_tests, 0b011_1000);
