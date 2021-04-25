use linux_embedded_hal::I2cdev;
use pcf857x::{Pcf8574, PinFlag, SlaveAddr};

fn main() {
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let address = SlaveAddr::default();
    let mut expander = Pcf8574::new(dev, address);
    let output_pin_status = 0b1010_1010;
    expander.set(output_pin_status).unwrap();

    let pins_to_be_read = PinFlag::P0 | PinFlag::P7;
    let input_status = expander.get(pins_to_be_read).unwrap();

    println!("Input pin status: {}", input_status);
}
