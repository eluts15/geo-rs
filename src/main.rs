use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub mod fetch;
pub mod gpio_input;

#[cfg(test)]
pub mod mock_gpio;

#[cfg(test)]
pub mod mock_pwm;

use fetch::fetch_with_tracker;
use geo_rs::GpsTracker;
use geo_rs::compass::heading_to_direction_8point;
use geo_rs::compass_sensor::CompassSensor;
use gpio_input::UserInterface;

const LOOKAHEAD_DISTANCE_M: f64 = 100.0;
const STATUS_UPDATE_INTERVAL_SECS: u64 = 1;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting GPS Navigation System...");

    let tracker = Arc::new(Mutex::new(GpsTracker::new()));
    let mut ui = UserInterface::new()?;

    // Initialize compass
    let mut compass = match CompassSensor::new() {
        Ok(c) => Some(c),
        Err(e) => {
            eprintln!("Compass not available: {} - GPS heading only", e);
            None
        }
    };

    initialize_system()?;
    start_gps_thread(Arc::clone(&tracker));
    wait_for_gps_fix(&tracker, &mut ui)?;

    run_main_loop(&tracker, &mut ui, &mut compass)?;

    Ok(())
}

fn initialize_system() -> Result<(), Box<dyn std::error::Error>> {
    println!("GPIO initialized:");
    println!("  Toggle Left:  GPIO 23");
    println!("  Toggle Right: GPIO 24");
    println!("  Servo PWM:    Disabled (testing compass only)");

    println!("\nHardware initialization complete.");
    Ok(())
}

fn start_gps_thread(tracker: Arc<Mutex<GpsTracker>>) {
    thread::spawn(move || {
        if let Err(e) = fetch_with_tracker(tracker) {
            eprintln!("\n❌ GPS error: {}", e);
            eprintln!("Check /dev/serial0 connection and permissions");
        }
    });

    // Give GPS thread time to open serial port
    thread::sleep(Duration::from_millis(100));
}

fn wait_for_gps_fix(
    tracker: &Arc<Mutex<GpsTracker>>,
    ui: &mut UserInterface,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting GPS data collection...");
    thread::sleep(Duration::from_millis(500)); // Let serial port open

    println!("Waiting for GPS fix...");
    println!("  (Make sure GPS antenna has clear view of sky)");

    let mut fix_attempts = 0;
    let start_time = std::time::Instant::now();

    loop {
        thread::sleep(Duration::from_millis(500));

        if let Ok(tracker_lock) = tracker.lock()
            && let Some(pos) = tracker_lock.get_current_position()
        {
            let elapsed = start_time.elapsed().as_secs();
            println!("\n✓ GPS fix acquired after {}s!", elapsed);
            println!("  Position: {}", pos);

            let gps_heading = tracker_lock.get_current_heading();
            let gps_speed = tracker_lock.get_current_speed();
            let num_sats = tracker_lock.get_num_satellites();
            let hdop = tracker_lock.get_current_hdop();

            drop(tracker_lock);

            if let Some(sats) = num_sats {
                println!("  Satellites: {}", sats);
            }

            if let Some(heading) = gps_heading {
                let (direction, _) = heading_to_direction_8point(heading);
                println!("  GPS heading: {:.1}° ({})", heading, direction);
                ui.update_gps_heading(heading);
            }

            if let Some(speed) = gps_speed {
                println!("  Speed: {:.2} knots", speed);
            }

            if let Some(hdop) = hdop {
                println!("  hdop: {:.2} ", hdop);
            }

            break;
        }

        fix_attempts += 1;

        // Show progress every 3 seconds
        if fix_attempts % 6 == 0 {
            let elapsed = start_time.elapsed().as_secs();
            print!("\r  Waiting for GPS fix... {}s", elapsed);
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }

        // Show reminder after 30 seconds
        if fix_attempts == 60 {
            println!("\n  ⚠ Still waiting for GPS fix...");
            println!("  • Check antenna connection");
            println!("  • Ensure clear view of sky");
            println!("  • Cold start can take 30s-5min");
        }
    }

    println!("\nMain control loop started.");
    println!("Use 3-way toggle to adjust heading (±5° increments)\n");

    Ok(())
}

fn run_main_loop(
    tracker: &Arc<Mutex<GpsTracker>>,
    ui: &mut UserInterface,
    compass: &mut Option<CompassSensor>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut last_status_update = std::time::Instant::now();

    loop {
        initialize_heading_if_needed(tracker, ui);
        handle_toggle_changes(tracker, ui)?;

        display_status_update(tracker, compass, ui, &mut last_status_update);

        thread::sleep(Duration::from_millis(20));
    }
}

fn initialize_heading_if_needed(tracker: &Arc<Mutex<GpsTracker>>, ui: &mut UserInterface) {
    if !ui.has_heading()
        && let Ok(tracker_lock) = tracker.lock()
        && let Some(gps_heading) = tracker_lock.get_current_heading()
    {
        let (direction, _) = heading_to_direction_8point(gps_heading);
        ui.update_gps_heading(gps_heading);
        println!(
            "✓ GPS heading acquired: {:.1}° ({}) | Offset: 0.0° (following GPS)",
            gps_heading, direction
        );
    }
}

fn handle_toggle_changes(
    tracker: &Arc<Mutex<GpsTracker>>,
    ui: &mut UserInterface,
) -> Result<(), Box<dyn std::error::Error>> {
    if ui.update()?
        && let Some(target_heading) = ui.get_heading()
    {
        let (direction, _) = heading_to_direction_8point(target_heading);

        if let Ok(tracker_lock) = tracker.lock() {
            if let Some(_pos) = tracker_lock.get_current_position()
                && let Some(vector) =
                    tracker_lock.get_vector_to_direction(target_heading, LOOKAHEAD_DISTANCE_M)
            {
                let target = vector.end_position();
                println!("  → Target heading: {:.1}° ({})", target_heading, direction);
                println!("     {}m ahead: {}", LOOKAHEAD_DISTANCE_M, target);
            }

            if let Some(gps_heading) = tracker_lock.get_current_heading() {
                let (gps_direction, _) = heading_to_direction_8point(gps_heading);
                println!("  → GPS heading: {:.1}° ({})", gps_heading, gps_direction);
            } else {
                println!("  → GPS heading: N/A (speed too low)");
            }
        }
    }

    Ok(())
}

fn display_status_update(
    tracker: &Arc<Mutex<GpsTracker>>,
    compass: &mut Option<CompassSensor>,
    ui: &mut UserInterface,
    last_status_update: &mut std::time::Instant,
) {
    if last_status_update.elapsed() >= Duration::from_secs(STATUS_UPDATE_INTERVAL_SECS) {
        if let Ok(tracker_lock) = tracker.lock()
            && let Some(pos) = tracker_lock.get_current_position()
        {
            println!("\n[Status Update]");
            println!("  Position: {}", pos);

            if let Some(target_heading) = ui.get_heading() {
                let (target_direction, _) = heading_to_direction_8point(target_heading);
                let offset = ui.get_heading_offset();
                println!(
                    "  Target heading: {:.1}° ({}) [Offset: {:.1}°]",
                    target_heading, target_direction, offset
                );
            } else {
                println!("  Target heading: Waiting for GPS...");
            }

            if let Some(num_sats) = tracker_lock.get_num_satellites() {
                println!("  Satellites: {}", num_sats);
            } else {
                println!("  Satellites: N/A");
            }

            if let Some(hdop) = tracker_lock.get_current_hdop() {
                println!("  HDOP: {:.2}", hdop);
            } else {
                println!("  HDOP: N/A");
            }

            // Show both GPS and compass headings
            let gps_heading = tracker_lock.get_current_heading();
            let compass_heading = compass.as_mut().and_then(|c| c.read_heading().ok());

            if let Some(heading) = gps_heading {
                let (gps_direction, _) = heading_to_direction_8point(heading);
                println!("  GPS heading: {:.1}° ({})", heading, gps_direction);
            } else {
                println!("  GPS heading: N/A (speed too low)");
            }

            if let Some(heading) = compass_heading {
                let (comp_direction, _) = heading_to_direction_8point(heading);
                println!("  Compass heading: {:.1}° ({})", heading, comp_direction);
            } else {
                println!("  Compass heading: N/A");
            }

            if let Some(speed) = tracker_lock.get_current_speed() {
                println!("  Speed: {:.2} knots", speed);
            }

            println!();
        }
        *last_status_update = std::time::Instant::now();
    }
}
