# GPS Navigation System for Raspberry Pi

A Rust-based GPS navigation system designed for marine navigation using a Raspberry Pi with BerryGPS-IMU-4 module. The system provides real-time position tracking, compass heading, and manual heading adjustment via GPIO toggles.

## Overview

This project combines GPS positioning with magnetometer-based compass readings to create a reliable navigation system. It's designed to run on a Raspberry Pi (tested on Pi 2) and provides continuous heading information whether the vessel is moving or stationary.

## Hardware Requirements

- **Raspberry Pi 2** (or compatible model)
- **BerryGPS-IMU-4** module
  - u-blox NEO GPS receiver
  - LIS3MDL magnetometer (compass)
  - LSM6DSL IMU (accelerometer/gyroscope)
- **GPIO toggles** (left/right) for manual heading adjustment
  - Toggle Left: GPIO 23
  - Toggle Right: GPIO 24
- **Optional:** PWM servo controller on GPIO 18 (currently disabled)

## Features

### Current Functionality

- ✅ **Real-time GPS positioning** - Continuous position updates via NMEA serial interface
- ✅ **Dual heading sources**
  - GPS heading (when moving >1-2 knots)
  - Magnetometer compass (works when stationary)
- ✅ **Calibrated compass** - Hard iron calibration removes magnetic interference
- ✅ **Manual heading adjustment** - ±5° increments via GPIO toggles
- ✅ **Status monitoring**
  - Satellite count and HDOP (position accuracy)
  - Position coordinates
  - GPS and compass headings with cardinal directions
- ✅ **Vector calculation** - Project target positions based on heading and distance

### Intended Future Functionality

- 🔲 **Autopilot servo control** - Automatic steering correction via PWM servo
- 🔲 **Route waypoint navigation** - Follow predefined GPS waypoints
- 🔲 **Current/drift compensation** - Compare GPS vs compass heading
- 🔲 **Enhanced stabilization** - PID control for smooth steering

## System Architecture

```
┌─────────────────────────────────────────────────────┐
│                  Main Control Loop                   │
├─────────────────────────────────────────────────────┤
│  • GPS Thread (NMEA parsing)                        │
│  • User Input (GPIO toggles)                        │
│  • Compass Reading (magnetometer)                   │
│  • Status Display (1 sec intervals)                 │
└─────────────────────────────────────────────────────┘
         │              │              │
         ▼              ▼              ▼
    ┌────────┐    ┌──────────┐   ┌─────────┐
    │  GPS   │    │ Compass  │   │  GPIO   │
    │ Module │    │ LIS3MDL  │   │ Toggles │
    └────────┘    └──────────┘   └─────────┘
```

## Installation

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Enable I2C and Serial
sudo raspi-config
# Navigate to: Interface Options -> I2C -> Enable
# Navigate to: Interface Options -> Serial Port
#   - Disable login shell: No
#   - Enable serial hardware: Yes
```

### Build

```bash
# Clone the repository
git clone <repository-url>
cd geo-rs

# Build the project
cargo build --release

# The binary will be in target/release/
```

## Configuration

### Compass Calibration

The magnetometer requires one-time calibration to account for hard iron distortion from the Raspberry Pi's electronics:

```bash
# Run the calibration utility
cargo run --bin calibrate

# Follow on-screen instructions:
# 1. Keep board level
# 2. Rotate slowly through 360°
# 3. Note the final X_OFFSET and Y_OFFSET values
# 4. Update these values in src/compass_sensor.rs
```

Current calibration constants (update for your specific setup, update `src/config.rs`, constants are located at the top of the file):
```rust
const X_OFFSET: f64 = -2776.0;  // Your calibrated value
const Y_OFFSET: f64 = 2556.0;   // Your calibrated value
const HEADING_OFFSET: f64 = 88.0;  // Location-specific correction
```

### GPS Configuration

The system expects GPS data on `/dev/serial0` at 9600 baud (default for u-blox NEO modules).

## Usage

### Running the Main Program

```bash
# Run the navigation system
cargo run --release

# Or run the compiled binary directly
sudo ./target/release/geo-rs
```

### Startup Sequence

1. **Hardware initialization** - Initializes GPIO pins and I2C devices
2. **GPS fix acquisition** - Waits for satellite lock (may take 30s-5min on cold start)
3. **Main control loop** - Continuous operation

### Manual Heading Adjustment

- **Toggle Left (GPIO 23)**: Decrease target heading by 5°
- **Toggle Right (GPIO 24)**: Increase target heading by 5°

The system displays the lookahead position (100m ahead on current heading) when you adjust.

### Status Display

Every second, the system displays:

```
[Status Update]
  Position: (48.056597°, -123.119772°)
  Satellites: 12
  HDOP: 0.88
  GPS heading: 245.3° (SW)
  Compass heading: 180.2° (S)
```

## Project Structure

```
src/
├── main.rs              # Main control loop and initialization
├── compass_sensor.rs    # LIS3MDL magnetometer interface
├── gps_tracker.rs       # GPS position and heading tracking
├── fetch.rs             # NMEA sentence parsing
├── gpio_input.rs        # GPIO toggle handling
├── position.rs          # GPS coordinate representation
├── vector.rs            # Heading vector calculations
└── calibrate.rs # Magnetometer calibration utility
```

## Technical Details

### GPS Heading vs Compass Heading

**GPS Heading:**
- Calculated from movement (change in position)
- Only available when moving (>1-2 knots)
- Shows direction of travel
- Unaffected by magnetic interference

**Compass Heading:**
- Read from magnetometer sensor
- Always available (even when stationary)
- Shows direction device is pointing
- Requires calibration for accuracy

**Use Cases:**
- **Stationary:** Only compass available
- **Moving:** Both available - difference indicates drift/current
- **Navigation:** GPS heading for course over ground, compass for vessel orientation

### Coordinate System

The BerryGPS-IMU-4 magnetometer axes:
- **X-axis:** Along board length (GPS antenna end)
- **Y-axis:** Across board width
- **Z-axis:** Perpendicular to board (upward)

**Orientation for North:**
Point the GPS antenna end of the board toward North (USB ports facing South).

### Calibration Math

Hard iron calibration removes constant magnetic offsets:

```
X_calibrated = X_raw - X_offset
Y_calibrated = Y_raw - Y_offset

Where:
X_offset = (X_min + X_max) / 2
Y_offset = (Y_min + Y_max) / 2
```

This centers the magnetometer readings, converting an ellipse to a circle for accurate 360° readings.

## Troubleshooting

### Garbage Data Read from /dev/serial0
- I ran into this issue and this seems to resolve it
[link](https://ozzmaker.com/forums/topic/nmea-unkown-msg46/)
```
 stty -F /dev/serial0 -echo
```

### GPS Not Getting Fix

- Ensure clear view of sky for antenna
- Wait 30s-5min for cold start acquisition
- Check `/dev/serial0` permissions: `sudo usermod -a -G dialout $USER`
- Verify connection: `cat /dev/serial0` (should show NMEA sentences)

### Compass Readings Inaccurate

- Run calibration routine
- Keep away from large metal objects and motors
- Ensure board is mounted level
- Check for electrical interference

### Compass Not Responding to Rotation

- Verify I2C is enabled: `sudo i2cdetect -y 1` (should show device at 0x1C)
- Check calibration offsets are applied
- Ensure board is level (tilt affects readings)

## Dependencies

- `rppal` - Raspberry Pi GPIO and I2C interface
- `nmea`  - External Library for parsing NMEA sentences
- `geo-rs` - Internal library for GPS calculations
- `geo-calibrate` - Tool for calibrating the compass

## Future Development

### Planned Features


1. **Autopilot Integration**
   - Re-enable servo control
   - PID controller for smooth steering
   - Configurable steering sensitivity
---

**Note:** This is a hobby/educational project. For critical marine navigation, always use certified equipment and traditional navigation :)

## Hardware
GPS Module (BerryGPS-IMU v4)
[link](https://ozzmaker.com/product/berrygps-imu/)

Antenna Module (CAM-M8C-0-10)
[link](https://www.mouser.com/ProductDetail/u-blox/CAM-M8C-0?qs=vEM7xhTegWh0Qdx4vzEerw%3D%3D)

GPS Antenna (ANT-105-SMA)
Active GPS Antenna
[link](TODO!)

