use rppal::i2c::I2c;
use std::error::Error;

const LIS3MDL_ADDR: u16 = 0x1C;

// Magnetometer calibration offsets (hard iron correction)
// Obtained from calibration: rotate board 360° and record min/max X,Y values
const X_OFFSET: f64 = -2776.0; // (X_min + X_max) / 2
const Y_OFFSET: f64 = 2556.0; // (Y_min + Y_max) / 2
const HEADING_OFFSET: f64 = 88.0; // Overall heading correction for this location

// LIS3MDL Register addresses
const WHO_AM_I: u8 = 0x0F;
const CTRL_REG1: u8 = 0x20;
const CTRL_REG2: u8 = 0x21;
const CTRL_REG3: u8 = 0x22;
const CTRL_REG4: u8 = 0x23;
const CTRL_REG5: u8 = 0x24;
const STATUS_REG: u8 = 0x27;
const OUT_X_L: u8 = 0x28;

pub struct CompassSensor {
    i2c: I2c,
}

impl CompassSensor {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut i2c = I2c::new()?;
        i2c.set_slave_address(LIS3MDL_ADDR)?;

        // Verify device ID (should be 0x3D for LIS3MDL)
        let who_am_i = i2c.smbus_read_byte(WHO_AM_I)?;
        if who_am_i != 0x3D {
            return Err(format!("Wrong device ID: 0x{:02X}, expected 0x3D", who_am_i).into());
        }

        // Initialize LIS3MDL magnetometer
        // CTRL_REG1: Temperature enabled, Ultra-high performance mode (X,Y), ODR = 80 Hz
        i2c.smbus_write_byte(CTRL_REG1, 0xFC)?;

        // CTRL_REG2: Full scale ±4 gauss
        i2c.smbus_write_byte(CTRL_REG2, 0x00)?;

        // CTRL_REG3: Continuous conversion mode
        i2c.smbus_write_byte(CTRL_REG3, 0x00)?;

        // CTRL_REG4: Ultra-high performance mode (Z-axis), little endian
        i2c.smbus_write_byte(CTRL_REG4, 0x0C)?;

        // CTRL_REG5: Block data update enabled
        i2c.smbus_write_byte(CTRL_REG5, 0x40)?;

        std::thread::sleep(std::time::Duration::from_millis(100));

        // Check if data is available
        let status = i2c.smbus_read_byte(STATUS_REG)?;
        if status == 0 {
            return Err("Magnetometer hardware not responding (no data ready)".into());
        }

        println!("✓ Compass (LIS3MDL) initialized");
        Ok(Self { i2c })
    }

    pub fn read_heading(&mut self) -> Result<f64, Box<dyn Error>> {
        // Wait for data to be ready
        let status = self.i2c.smbus_read_byte(STATUS_REG)?;
        if status & 0x08 == 0 {
            return Err("Magnetometer data not ready".into());
        }

        // Read 6 bytes starting from OUT_X_L (auto-increment enabled)
        let mut data = [0u8; 6];
        for (i, item) in data.iter_mut().enumerate() {
            *item = self.i2c.smbus_read_byte(OUT_X_L + i as u8)?;
        }

        // Convert to signed 16-bit values (little endian)
        let x_raw = i16::from_le_bytes([data[0], data[1]]) as f64;
        let y_raw = i16::from_le_bytes([data[2], data[3]]) as f64;

        // Apply hard iron calibration (center the readings)
        let x = x_raw - X_OFFSET;
        let y = y_raw - Y_OFFSET;

        // Calculate heading using calibrated values
        let raw_heading = y.atan2(x).to_degrees();

        // Apply final heading offset for location
        let calibrated_heading = raw_heading + HEADING_OFFSET;

        // Normalize to 0-360 range
        Ok((calibrated_heading + 360.0) % 360.0)
    }

    /// Read raw magnetometer X, Y, Z values (for calibration)
    pub fn read_raw_magnetometer(&mut self) -> Result<(f64, f64), Box<dyn Error>> {
        // Wait for data to be ready
        let status = self.i2c.smbus_read_byte(STATUS_REG)?;
        if status & 0x08 == 0 {
            return Err("Magnetometer data not ready".into());
        }

        // Read 6 bytes starting from OUT_X_L (auto-increment enabled)
        let mut data = [0u8; 6];
        for (i, item) in data.iter_mut().enumerate() {
            *item = self.i2c.smbus_read_byte(OUT_X_L + i as u8)?;
        }

        // Convert to signed 16-bit values (little endian)
        let x = i16::from_le_bytes([data[0], data[1]]) as f64;
        let y = i16::from_le_bytes([data[2], data[3]]) as f64;

        Ok((x, y))
    }
}
