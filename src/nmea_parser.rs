use std::fs::File;
use std::io::{self, BufRead, BufReader};

use geo_rs::GpsTracker;
use nmea::Nmea;

pub fn parse_nmea() -> io::Result<()> {
    //let file = File::open("/dev/serial0")?;
    // TODO: Temporary test.
    let test_file_path =
        "/home/ethanl/personal/learning_projects/rust/embedded/pi/geo-rs/nmea-sample.txt";

    let file = File::open(test_file_path)?;
    let reader = BufReader::new(file);

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

pub fn run() -> io::Result<()> {
    println!("Opening /dev/serial0...");
    println!("Reading GPS data from serial port. Press Ctrl+C to exit.\n");

    //let file = File::open("/dev/serial0")?;
    let test_file_path =
        "/home/ethanl/personal/learning_projects/rust/embedded/pi/geo-rs/nmea-sample.txt";

    let file = File::open(test_file_path)?;
    let reader = BufReader::new(file);

    let mut nmea = Nmea::default();
    let mut tracker = GpsTracker::new();

    for line in reader.lines() {
        match line {
            Ok(content) => {
                let trimmed = content.trim();
                if trimmed.is_empty() {
                    continue;
                }

                // Try to parse the NMEA sentence
                if let Ok(_) = nmea.parse(trimmed) {
                    // Update tracker with latest GPS data
                    if let (Some(lat), Some(lon)) = (nmea.latitude, nmea.longitude) {
                        tracker.update_position(lat, lon);
                    }

                    if let Some(heading) = nmea.true_course {
                        println!("  Heading updated: {:.1}°", heading);
                        tracker.update_heading(heading.into());
                    }

                    if let Some(speed) = nmea.speed_over_ground {
                        println!("  Speed: {:.2} knots", speed);
                        tracker.update_speed(speed.into());
                    }

                    // Print current position and forward vector
                    if let Some(pos) = tracker.get_current_position() {
                        println!("Position: {}", pos);

                        if let Some(vector) = tracker.get_forward_vector(100.0) {
                            println!("  Forward: {}", vector);
                        } else {
                            println!("  No vector (need heading data - try moving!)");
                        }

                        // Example: Show vector in cardinal directions
                        if let Some(north) = tracker.get_vector_in_direction(0.0, 100.0) {
                            println!("  North:   {}", north);
                        }

                        println!();
                    }
                }
            }
            Err(e) => eprintln!("Error reading line: {}", e),
        }
    }

    Ok(())
}
