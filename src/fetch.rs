use std::io;
use std::sync::{Arc, Mutex};

use crate::GpsTracker;

// TODO: Add NMEA sentence validation
// pub fn validate_sentence() {}

pub fn fetch_with_tracker(tracker: Arc<Mutex<GpsTracker>>) -> io::Result<()> {
    use nmea::Nmea;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    println!("Opening /dev/serial0...");

    let file = match File::open("/dev/serial0") {
        Ok(f) => {
            println!("✓ Serial port opened successfully");
            f
        }
        Err(e) => {
            eprintln!("❌ Failed to open /dev/serial0: {}", e);
            eprintln!("  • Check if GPS is connected");
            eprintln!("  • Verify user is in 'dialout' group: sudo usermod -a -G dialout $USER");
            eprintln!("  • Check permissions: ls -l /dev/serial0");
            return Err(e);
        }
    };

    let reader = BufReader::new(file);
    let mut nmea = Nmea::default();
    let mut sentence_count = 0;

    for line in reader.lines() {
        match line {
            Ok(content) => {
                let trimmed = content.trim();
                if trimmed.is_empty() {
                    continue;
                }

                if nmea.parse(trimmed).is_ok()
                    && let Ok(mut tracker_lock) = tracker.lock()
                {
                    sentence_count += 1;

                    // Log first valid sentence
                    if sentence_count == 1 {
                        println!("✓ Receiving GPS data");
                    }

                    if let (Some(lat), Some(lon)) = (nmea.latitude, nmea.longitude) {
                        tracker_lock.update_position(lat, lon);
                    }

                    if let Some(heading) = nmea.true_course {
                        tracker_lock.update_heading(heading.into());
                    }

                    if let Some(num_sats) = nmea.num_of_fix_satellites {
                        tracker_lock.update_satellites(num_sats.try_into().unwrap_or(0));
                    }

                    if let Some(speed) = nmea.speed_over_ground {
                        tracker_lock.update_speed(speed.into());
                    }
                }
            }
            Err(e) => eprintln!("Error reading line: {}", e),
        }
    }

    Ok(())
}
