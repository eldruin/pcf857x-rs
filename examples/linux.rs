extern crate embedded_hal;
extern crate linux_embedded_hal;
extern crate pcf857x;

use linux_embedded_hal::I2cdev;
use pcf857x::{PCF8574, SlaveAddr, PinFlag};

fn main() {
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let mut expander = PCF8574::new(dev, SlaveAddr::default());
    let output_pin_status = 0b1010_1010;
    expander.set(output_pin_status).unwrap();
    
    let mask_of_pins_to_be_read = PinFlag::P0 | PinFlag::P7;
    let read_input_pin_status = expander.get(&mask_of_pins_to_be_read).unwrap();

    println!("Input pin status: {}", read_input_pin_status);
}
