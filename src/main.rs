use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub mod fetch;
pub mod gpio_input;

// This module is only compiled during testing.
#[cfg(test)]
pub mod mock_gpio;

use fetch::fetch_with_tracker;
use geo_rs::GpsTracker;
use geo_rs::compass::heading_to_azimuth_8point;
use gpio_input::UserInterface;

// Usage:
//  Start: Heading set from GPS (e.g., 84.5° East)
//  → RIGHT: Now 89.5° (adjust +5° from current) - travel straight on this line
//  → RIGHT: Now 94.5° (adjust +5° from current) - travel straight on this NEW line
//  ● NEUTRAL: Stay at 94.5° - continue straight
//  ← LEFT: Now 89.5° (adjust -5° from current) - travel straight on this line
//  ● NEUTRAL: Stay at 89.5° - continue straight
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting GPS Navigation System...");

    // Shared GPS tracker between threads
    let tracker = Arc::new(Mutex::new(GpsTracker::new()));
    let tracker_clone = Arc::clone(&tracker);

    // Initialize GPIO interface with default pins (23, 24)
    let mut ui = UserInterface::new()?;

    println!("GPIO initialized:");
    println!("  Toggle Left:  GPIO 23");
    println!("  Toggle Right: GPIO 24");
    println!("\nStarting GPS data collection...");

    // Start GPS tracking in a separate thread
    thread::spawn(move || {
        if let Err(e) = fetch_with_tracker(tracker_clone) {
            eprintln!("GPS error: {}", e);
        }
    });

    println!("Waiting for GPS fix...");
    let mut has_fix = false;
    let mut fix_attempts = 0;

    while !has_fix {
        thread::sleep(Duration::from_millis(500));

        if let Ok(tracker_lock) = tracker.lock()
            && let Some(pos) = tracker_lock.get_current_position()
        {
            has_fix = true;
            println!("\n✓ GPS fix acquired!");
            println!("  Position: {}", pos);

            let gps_heading = tracker_lock.get_current_heading();
            let gps_speed = tracker_lock.get_current_speed();

            // Release lock before calling ui methods
            drop(tracker_lock);

            if let Some(heading) = gps_heading {
                let (direction, _) = heading_to_azimuth_8point(heading);
                println!("  GPS heading: {:.1}° ({})", heading, direction);
                // Set initial heading from GPS - this is the actual compass direction
                ui.set_heading(heading);
            }

            if let Some(speed) = gps_speed {
                println!("  Speed: {:.2} knots", speed);
            }
        }

        fix_attempts += 1;
        if fix_attempts % 6 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }
    }

    println!("\nMain control loop started.");
    println!("Use 3-way toggle to adjust heading (±5° increments)\n");

    let mut last_status_update = std::time::Instant::now();

    // Main control loop
    loop {
        // Initialize heading from GPS if not set yet
        if !ui.has_heading()
            && let Ok(tracker_lock) = tracker.lock()
            && let Some(gps_heading) = tracker_lock.get_current_heading()
        {
            let (direction, _) = heading_to_azimuth_8point(gps_heading);
            ui.set_heading(gps_heading);
            println!(
                "✓ Target heading initialized: {:.1}° ({})",
                gps_heading, direction
            );
        }
        // Update GPIO and check for toggle changes
        if ui.update()? {
            // Toggle position changed and we have a heading
            if let Some(target_heading) = ui.get_heading() {
                let (direction, _) = heading_to_azimuth_8point(target_heading);

                // Get current GPS data
                if let Ok(tracker_lock) = tracker.lock() {
                    if let Some(_pos) = tracker_lock.get_current_position() {
                        // Calculate target position 100m ahead at new heading
                        if let Some(vector) =
                            tracker_lock.get_vector_in_direction(target_heading, 100.0)
                        {
                            let target = vector.end_position();
                            println!("  → Target heading: {:.1}° ({})", target_heading, direction);
                            println!("     100m ahead: {}", target);
                        }
                    }

                    // Show current GPS heading if available
                    if let Some(gps_heading) = tracker_lock.get_current_heading() {
                        let (gps_dir, _) = heading_to_azimuth_8point(gps_heading);
                        println!("  → GPS heading: {:.1}° ({})", gps_heading, gps_dir);
                    } else {
                        println!("  → GPS heading: N/A (speed too low)");
                    }
                }
            }
        } else {
            // Periodically show status updates every 5 seconds when idle
            if last_status_update.elapsed() >= Duration::from_secs(5) {
                if let Ok(tracker_lock) = tracker.lock()
                    && let Some(pos) = tracker_lock.get_current_position()
                {
                    println!("\n[Status Update]");
                    println!("  Position: {}", pos);

                    if let Some(target_heading) = ui.get_heading() {
                        let (target_dir, _) = heading_to_azimuth_8point(target_heading);
                        println!("  Target heading: {:.1}° ({})", target_heading, target_dir);
                    } else {
                        println!("  Target heading: Waiting for GPS...");
                    }

                    // Add satellite count
                    if let Some(num_sats) = tracker_lock.get_num_satellites() {
                        println!("  Satellites: {}", num_sats);
                    } else {
                        println!("  Satellites: N/A");
                    }

                    if let Some(gps_heading) = tracker_lock.get_current_heading() {
                        let (gps_dir, _) = heading_to_azimuth_8point(gps_heading);
                        println!("  GPS heading: {:.1}° ({})", gps_heading, gps_dir);
                    } else {
                        println!("  GPS heading: N/A (speed too low)");
                    }

                    if let Some(speed) = tracker_lock.get_current_speed() {
                        println!("  Speed: {:.2} knots\n", speed);
                    }

                    println!(); // Empty line after status
                }
                last_status_update = std::time::Instant::now();
            }

            // Periodically sync with GPS heading when in neutral position
            if ui.get_toggle_position() == gpio_input::SwitchPosition::Neutral
                && let Ok(tracker_lock) = tracker.lock()
                && let Some(gps_heading) = tracker_lock.get_current_heading()
                && let Some(current) = ui.get_heading()
            {
                let diff = (gps_heading - current).abs();

                // Sync if difference is significant
                if diff > 10.0 && diff < 350.0 {
                    let (direction, _) = heading_to_azimuth_8point(gps_heading);
                    ui.set_heading(gps_heading);
                    println!(
                        "Synced with GPS heading: {:.1}° ({})",
                        gps_heading, direction
                    );
                }
            }
        }

        // Debounce delay
        thread::sleep(Duration::from_millis(20));
    }
}
