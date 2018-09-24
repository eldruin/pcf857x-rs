extern crate pcf857x;
extern crate embedded_hal_mock as hal;
use pcf857x::{PCF8574, PCF8574A, SlaveAddr, PinFlag, Error};

macro_rules! pcf8574_tests {
    ($device_name:ident, $test_mod_name:ident, $default_address:expr) => {
        mod $test_mod_name {
            use super::*;
            
            fn setup<'a>(data: &'a[u8]) -> $device_name<hal::I2cMock<'a>> {
                let mut dev = hal::I2cMock::new();
                dev.set_read_data(&data);
                $device_name::new(dev, SlaveAddr::default())
            }

            fn check_sent_data(expander: $device_name<hal::I2cMock>, data: &[u8]) {
                let dev = expander.destroy();
                assert_eq!(dev.get_last_address(), Some($default_address));
                assert_eq!(dev.get_write_data(), &data[..]);
            }

            fn check_nothing_was_sent(expander: $device_name<hal::I2cMock>) {
                let dev = expander.destroy();
                assert!(dev.get_last_address().is_none());
                assert!(dev.get_write_data().is_empty());
            }


            #[test]
            fn can_set_output_values() {
                let status = 0b1010_1010;
                let mut expander = setup(&[0]);
                expander.set(status).unwrap();
                check_sent_data(expander, &[status]);
            }

            #[test]
            fn read_wrong_pin_flag_returns_error() {
                let mut expander = setup(&[0]);
                let mask = PinFlag::P0 | PinFlag::P17;
                match expander.get(&mask) {
                    Err(Error::InvalidInputData) => (),
                    _ => panic!()
                }
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
            fn empty_array_read_does_nothing() {
                let mut expander = setup(&[0xAB, 0xCD]);
                let mask = PinFlag::P0 | PinFlag::P7;
                expander.read_array(&mask, &mut []).unwrap();
                check_nothing_was_sent(expander);
            }

            #[test]
            fn read_multiple_words_but_wrong_pin_flag_returns_error() {
                let mut data = [0; 2];
                let mut expander = setup(&[0xAB, 0xCD]);
                let mask = PinFlag::P0 | PinFlag::P17;
                match expander.read_array(&mask, &mut data) {
                    Err(Error::InvalidInputData) => (),
                    _ => panic!()
                }
            }
            pcf8574_pin_test!(p0,   1);
            pcf8574_pin_test!(p1,   2);
            pcf8574_pin_test!(p2,   4);
            pcf8574_pin_test!(p3,   8);
            pcf8574_pin_test!(p4,  16);
            pcf8574_pin_test!(p5,  32);
            pcf8574_pin_test!(p6,  64);
            pcf8574_pin_test!(p7, 128);
        }
    }
}

macro_rules! pcf8574_pin_test {
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
                check_sent_data(expander, &[$value]);
            }

            #[test]
            fn can_split_and_set_low() {
                let mut expander = setup(&[0]);
                expander.set(0b1111_1111).unwrap();
                {
                  let mut parts = expander.split();
                  parts.$px.set_low();
                }
                check_sent_data(expander, &[0b1111_1111 & !$value]);
            }

            #[cfg(feature = "unproven")]
            #[test]
            fn can_split_and_get_is_high() {
                let expander = setup(&[$value]);
                {
                  let parts = expander.split();
                  assert!(parts.$px.is_high());
                }
                check_sent_data(expander, &[$value]);
            }

            #[cfg(feature = "unproven")]
            #[test]
            fn can_split_and_get_is_low() {
                let expander = setup(&[!$value]);
                {
                  let parts = expander.split();
                  assert!(parts.$px.is_low());
                }
                check_sent_data(expander, &[$value]);
            }
        }
    }
}

pcf8574_tests!(PCF8574,  pcf8574_tests,  0b010_0000);
pcf8574_tests!(PCF8574A, pcf8574a_tests, 0b011_1000);
