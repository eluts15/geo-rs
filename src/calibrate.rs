use geo_rs::compass_sensor::CompassSensor;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║     Magnetometer Calibration Tool                    ║");
    println!("╚══════════════════════════════════════════════════════╝\n");

    println!("Instructions:");
    println!("1. Keep the board LEVEL (horizontal)");
    println!("2. Slowly rotate the board through a FULL 360° circle");
    println!("3. Take at least 30 seconds to complete the rotation");
    println!("4. Try to rotate smoothly at constant speed");
    println!("5. Press Ctrl+C when done\n");

    println!("Starting in 5 seconds...\n");
    thread::sleep(Duration::from_secs(5));

    let mut compass = CompassSensor::new()?;

    let mut x_min = f64::MAX;
    let mut x_max = f64::MIN;
    let mut y_min = f64::MAX;
    let mut y_max = f64::MIN;

    let mut sample_count = 0;

    println!("Collecting samples... (ROTATE NOW!)");
    println!(
        "\n{:^8} | {:^20} | {:^20} | {:^20}",
        "Sample", "X Range", "Y Range", "Calculated Offsets"
    );
    println!("{:-<8}-+-{:-<20}-+-{:-<20}-+-{:-<20}", "", "", "", "");

    loop {
        if let Ok((x, y)) = compass.read_raw_magnetometer() {
            // Update min/max
            if x < x_min {
                x_min = x;
            }
            if x > x_max {
                x_max = x;
            }
            if y < y_min {
                y_min = y;
            }
            if y > y_max {
                y_max = y;
            }

            sample_count += 1;

            // Calculate offsets (center of circle)
            let x_offset = (x_min + x_max) / 2.0;
            let y_offset = (y_min + y_max) / 2.0;

            // Print update every 10 samples
            if sample_count % 10 == 0 {
                println!(
                    "{:^8} | {:>7.0} to {:>7.0} | {:>7.0} to {:>7.0} | X: {:>7.0}  Y: {:>7.0}",
                    sample_count, x_min, x_max, y_min, y_max, x_offset, y_offset
                );
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}
