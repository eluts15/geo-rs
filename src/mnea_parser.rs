use nmea::Nmea;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

pub fn parse_nmea() -> io::Result<()> {
    println!("Opening /dev/serial0...");

    //let file = File::open("/dev/serial0")?;

    // TODO: Temporary test.
    let test_file_path =
        "/home/ethanl/personal/learning_projects/rust/embedded/pi/geo-rs/nmea-sample.txt";

    let file = File::open(test_file_path)?;
    let reader = BufReader::new(file);

    println!("Reading GPS data from serial port. Press Ctrl+C to exit.\n");

    let mut nmea = Nmea::default();

    for line in reader.lines() {
        match line {
            Ok(content) => {
                let trimmed = content.trim();
                if trimmed.is_empty() {
                    continue;
                }

                // Print raw NMEA sentence
                println!("Raw: {}", trimmed);

                // Try to parse and accumulate NMEA data
                match nmea.parse(trimmed) {
                    Ok(_) => {
                        // Display parsed GPS information
                        if let Some(lat) = nmea.latitude
                            && let Some(lon) = nmea.longitude
                        {
                            println!("  → Position: Lat {:.6}°, Lon {:.6}°", lat, lon);
                        }

                        if let Some(speed) = nmea.speed_over_ground {
                            println!("  → Speed: {:.2} knots", speed);
                        }

                        if let Some(course) = nmea.true_course {
                            println!("  → Course: {:.1}°", course);
                        }

                        if let Some(num_sats) = nmea.num_of_fix_satellites {
                            println!("  → Satellites used: {}", num_sats);
                        }

                        if let Some(hdop) = nmea.hdop {
                            println!("  → HDOP: {:.2}", hdop);
                        }

                        if let Some(fix_time) = nmea.fix_time {
                            println!("  → Fix time: {}", fix_time.format("%H:%M:%S"));
                        }

                        if let Some(fix_type) = nmea.fix_type {
                            println!("  → Fix type: {:?}", fix_type);
                        }
                    }
                    Err(e) => {
                        println!("  → Parse error: {}", e);
                    }
                }
                println!();
            }
            Err(e) => eprintln!("Error reading line: {}", e),
        }
    }

    Ok(())
}
