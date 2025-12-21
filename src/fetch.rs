use std::fs::File;
use std::io::{self, BufRead, BufReader};

use geo_rs::GpsTracker;
use nmea::Nmea;

pub fn fetch() -> io::Result<()> {
    println!("Opening /dev/serial0...");
    let file = File::open("/dev/serial0")?;
    let reader = BufReader::new(file);

    println!("Reading GPS data from serial port. Press Ctrl+C to exit.\n");

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
                if nmea.parse(trimmed).is_ok() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    #[test]
    fn test_fetch_from_file() -> io::Result<()> {
        // Read from the test GPS data file with valid checksums
        let test_file_path =
            "/home/ethanl/personal/learning_projects/rust/embedded/pi/geo-rs/test_gps.txt";
        let file = File::open(test_file_path)?;
        let reader = BufReader::new(file);

        let mut nmea = Nmea::default();
        let mut tracker = GpsTracker::new();

        let mut position_updated = false;
        let mut heading_updated = false;
        let mut valid_sentences = 0;
        let mut parse_errors = 0;

        for line in reader.lines() {
            match line {
                Ok(content) => {
                    let trimmed = content.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    match nmea.parse(trimmed) {
                        Ok(_) => {
                            valid_sentences += 1;

                            // Update tracker with position
                            if let (Some(lat), Some(lon)) = (nmea.latitude, nmea.longitude) {
                                tracker.update_position(lat, lon);
                                position_updated = true;

                                println!("Position: {:.6}°, {:.6}°", lat, lon);
                            }

                            // Update tracker with heading
                            if let Some(heading) = nmea.true_course {
                                tracker.update_heading(heading.into());
                                heading_updated = true;

                                println!("Heading: {:.1}°", heading);
                            }

                            // Update speed
                            if let Some(speed) = nmea.speed_over_ground {
                                tracker.update_speed(speed.into());
                                println!("Speed: {:.2} knots", speed);
                            }
                        }
                        Err(e) => {
                            parse_errors += 1;
                            println!("Parse error on line '{}': {}", trimmed, e);
                        }
                    }
                }
                Err(e) => eprintln!("Error reading line: {}", e),
            }
        }

        println!("\nTest Results:");
        println!("Valid NMEA sentences parsed: {}", valid_sentences);
        println!("Parse errors: {}", parse_errors);
        println!("Position updated: {}", position_updated);
        println!("Heading updated: {}", heading_updated);

        // Verify we parsed valid data
        assert!(
            valid_sentences > 0,
            "Should have parsed at least one valid NMEA sentence"
        );
        assert!(position_updated, "Should have updated position");
        assert!(heading_updated, "Should have updated heading");

        // Test vector creation
        let pos = tracker.get_current_position();
        assert!(pos.is_some(), "Tracker should have position");

        if let Some(position) = pos {
            println!("\nFinal Position: {}", position);

            // Test creating vectors in all cardinal directions
            let north = tracker.get_vector_in_direction(0.0, 100.0);
            assert!(north.is_some(), "Should create north vector");
            if let Some(v) = north {
                println!("North 100m: {}", v);
            }

            let east = tracker.get_vector_in_direction(90.0, 100.0);
            if let Some(v) = east {
                println!("East 100m: {}", v);
            }

            // Test forward vector
            let forward = tracker.get_forward_vector(100.0);
            assert!(forward.is_some(), "Should create forward vector");
            if let Some(v) = forward {
                println!("Forward 100m: {}", v);
            }
        }

        Ok(())
    }

    #[test]
    fn test_parse_individual_sentences() -> io::Result<()> {
        let test_file_path =
            "/home/ethanl/personal/learning_projects/rust/embedded/pi/geo-rs/test_gps.txt";
        let file = File::open(test_file_path)?;
        let reader = BufReader::new(file);

        let mut gga_count = 0;
        let mut rmc_count = 0;
        let mut gsa_count = 0;
        let mut gsv_count = 0;

        for line in reader.lines().map_while(Result::ok) {
            let trimmed = line;
            if trimmed.contains("GGA") {
                gga_count += 1;
            } else if trimmed.contains("RMC") {
                rmc_count += 1;
            } else if trimmed.contains("GSA") {
                gsa_count += 1;
            } else if trimmed.contains("GSV") {
                gsv_count += 1;
            }
        }

        println!("NMEA Sentence Types:");
        println!("  GGA (Position): {}", gga_count);
        println!("  RMC (Recommended Minimum): {}", rmc_count);
        println!("  GSA (DOP and Active Satellites): {}", gsa_count);
        println!("  GSV (Satellites in View): {}", gsv_count);

        // Just verify the file has some data
        let total = gga_count + rmc_count + gsa_count + gsv_count;
        assert!(
            total > 0,
            "Should have found some NMEA sentences in test file"
        );

        Ok(())
    }
}
