use rppal::i2c::I2c;
use std::error::Error;

const LSM303D_ADDR: u16 = 0x1C;

// const CTRL0: u8 = 0x1F;
// const CTRL1: u8 = 0x20;
// const CTRL2: u8 = 0x21;
// const CTRL5: u8 = 0x24;
// const CTRL6: u8 = 0x25;
// const CTRL7: u8 = 0x26;
// const OUT_X_L_M: u8 = 0x08;

pub struct CompassSensor {
    i2c: I2c,
}

impl CompassSensor {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut i2c = I2c::new()?;
        i2c.set_slave_address(LSM303D_ADDR)?;

        // Try to initialize magnetometer
        i2c.smbus_write_byte(0x20, 0x57)?;
        i2c.smbus_write_byte(0x24, 0xF0)?;
        i2c.smbus_write_byte(0x26, 0x00)?;
        std::thread::sleep(std::time::Duration::from_millis(200));

        // Check if responding
        let status = i2c.smbus_read_byte(0x07)?;
        if status == 0 {
            return Err("Magnetometer hardware not responding (likely disabled on board)".into());
        }

        println!("✓ Compass (LSM303D) initialized");
        Ok(Self { i2c })
    }

    pub fn read_heading(&mut self) -> Result<f64, Box<dyn Error>> {
        let mut data = [0u8; 6];
        for (i, item) in data.iter_mut().enumerate() {
            *item = self.i2c.smbus_read_byte(0x08 + i as u8)?;
        }

        let x = i16::from_le_bytes([data[0], data[1]]) as f64;
        let y = i16::from_le_bytes([data[2], data[3]]) as f64;

        let heading = y.atan2(x).to_degrees();
        Ok((heading + 360.0) % 360.0)
    }
}
