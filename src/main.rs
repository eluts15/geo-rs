use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub mod fetch;
pub mod gpio_input;

use fetch::fetch_with_tracker;
use geo_rs::GpsTracker;
use gpio_input::UserInterface;

// Usage:
//  Start: Heading 0° (North)
//  → RIGHT: Now 5° (slightly northeast) - travel straight on this line
//  → RIGHT: Now 10° (more east) - travel straight on this NEW line
//  ● NEUTRAL: Stay at 10° - continue straight
//  ← LEFT: Now 5° (back toward north) - travel straight on this line
//  ● NEUTRAL: Stay at 5° - continue straight

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
                println!("  GPS heading: {:.1}°", heading);
                // Set initial heading from GPS
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
        // Update GPIO and check for toggle changes
        if ui.update()? {
            // Toggle position changed
            let target_heading = ui.get_heading();

            // Get current GPS data
            if let Ok(tracker_lock) = tracker.lock() {
                if let Some(_pos) = tracker_lock.get_current_position() {
                    // Calculate target position 100m ahead at new heading
                    if let Some(vector) =
                        tracker_lock.get_vector_in_direction(target_heading, 100.0)
                    {
                        let target = vector.end_position();
                        println!("  → Target (100m @ {:.1}°): {}", target_heading, target);
                    }
                }

                // Show current GPS heading if available
                if let Some(gps_heading) = tracker_lock.get_current_heading() {
                    println!("  → GPS heading: {:.1}°", gps_heading);
                }
            }
        } else {
            // Periodically show status updates every 5 seconds when idle
            if last_status_update.elapsed() >= Duration::from_secs(5) {
                if let Ok(tracker_lock) = tracker.lock()
                    && let Some(pos) = tracker_lock.get_current_position()
                {
                    println!("\n[Status]");
                    println!("  Position: {}", pos);
                    println!("  Target heading: {:.1}°", ui.get_heading());

                    if let Some(gps_heading) = tracker_lock.get_current_heading() {
                        println!("  GPS heading: {:.1}°", gps_heading);
                    }

                    if let Some(speed) = tracker_lock.get_current_speed() {
                        println!("  Speed: {:.2} knots\n", speed);
                    }
                }
                last_status_update = std::time::Instant::now();
            }

            // Periodically sync with GPS heading when in neutral position
            if ui.get_toggle_position() == gpio_input::SwitchPosition::Neutral
                && let Ok(tracker_lock) = tracker.lock()
                && let Some(gps_heading) = tracker_lock.get_current_heading()
            {
                let current = ui.get_heading();
                let diff = (gps_heading - current).abs();

                // Sync if difference is significant
                if diff > 10.0 && diff < 350.0 {
                    ui.set_heading(gps_heading);
                    println!("Synced with GPS heading: {:.1}°", gps_heading);
                }
            }
        }

        // Debounce delay
        thread::sleep(Duration::from_millis(20));
    }
}
