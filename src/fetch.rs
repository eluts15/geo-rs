use geo_rs::GpsTracker;
use std::io;
use std::sync::{Arc, Mutex};

pub fn fetch_with_tracker(tracker: Arc<Mutex<GpsTracker>>) -> io::Result<()> {
    use nmea::Nmea;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    println!("Opening /dev/serial0...");
    let file = File::open("/dev/serial0")?;
    let reader = BufReader::new(file);

    let mut nmea = Nmea::default();

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
                    // Update tracker with latest GPS data
                    if let (Some(lat), Some(lon)) = (nmea.latitude, nmea.longitude) {
                        tracker_lock.update_position(lat, lon);
                    }

                    if let Some(heading) = nmea.true_course {
                        tracker_lock.update_heading(heading.into());
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
