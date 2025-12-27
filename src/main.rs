use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub mod fetch;
pub mod gpio_input;

#[cfg(test)]
pub mod mock_gpio;

use fetch::fetch_with_tracker;
use geo_rs::GpsTracker;
use geo_rs::compass::heading_to_azimuth_8point;
use gpio_input::UserInterface;

const LOOKAHEAD_DISTANCE_M: f64 = 100.0;
const STATUS_UPDATE_INTERVAL_SECS: u64 = 5;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting GPS Navigation System...");

    let tracker = Arc::new(Mutex::new(GpsTracker::new()));
    let mut ui = UserInterface::new()?;

    initialize_system();
    start_gps_thread(Arc::clone(&tracker));
    wait_for_gps_fix(&tracker, &mut ui)?;

    run_main_loop(&tracker, &mut ui)?;

    Ok(())
}

fn initialize_system() {
    println!("GPIO initialized:");
    println!("  Toggle Left:  GPIO 23");
    println!("  Toggle Right: GPIO 24");
    println!("\nHardware initialization complete.");
}

fn start_gps_thread(tracker: Arc<Mutex<GpsTracker>>) {
    println!("Starting GPS data collection...");
    thread::spawn(move || {
        if let Err(e) = fetch_with_tracker(tracker) {
            eprintln!("GPS error: {}", e);
        }
    });
}

fn wait_for_gps_fix(
    tracker: &Arc<Mutex<GpsTracker>>,
    ui: &mut UserInterface,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Waiting for GPS fix...");
    let mut fix_attempts = 0;

    loop {
        thread::sleep(Duration::from_millis(500));

        if let Ok(tracker_lock) = tracker.lock()
            && let Some(pos) = tracker_lock.get_current_position()
        {
            println!("\n✓ GPS fix acquired!");
            println!("  Position: {}", pos);

            let gps_heading = tracker_lock.get_current_heading();
            let gps_speed = tracker_lock.get_current_speed();
            drop(tracker_lock);

            if let Some(heading) = gps_heading {
                let (direction, _) = heading_to_azimuth_8point(heading);
                println!("  GPS heading: {:.1}° ({})", heading, direction);
                ui.set_heading(heading);
            }

            if let Some(speed) = gps_speed {
                println!("  Speed: {:.2} knots", speed);
            }

            break;
        }

        fix_attempts += 1;
        if fix_attempts % 6 == 0 {
            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }
    }

    println!("\nMain control loop started.");
    println!("Use 3-way toggle to adjust heading (±5° increments)\n");

    Ok(())
}

fn run_main_loop(
    tracker: &Arc<Mutex<GpsTracker>>,
    ui: &mut UserInterface,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut last_status_update = std::time::Instant::now();

    loop {
        initialize_heading_if_needed(tracker, ui);
        handle_toggle_changes(tracker, ui)?;
        display_status_update(tracker, ui, &mut last_status_update);

        thread::sleep(Duration::from_millis(20));
    }
}

fn initialize_heading_if_needed(tracker: &Arc<Mutex<GpsTracker>>, ui: &mut UserInterface) {
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
}

fn handle_toggle_changes(
    tracker: &Arc<Mutex<GpsTracker>>,
    ui: &mut UserInterface,
) -> Result<(), Box<dyn std::error::Error>> {
    if ui.update()?
        && let Some(target_heading) = ui.get_heading()
    {
        let (direction, _) = heading_to_azimuth_8point(target_heading);

        if let Ok(tracker_lock) = tracker.lock() {
            if let Some(_pos) = tracker_lock.get_current_position() {
                if let Some(vector) =
                    tracker_lock.get_vector_in_direction(target_heading, LOOKAHEAD_DISTANCE_M)
                {
                    let target = vector.end_position();
                    println!("  → Target heading: {:.1}° ({})", target_heading, direction);
                    println!("     {}m ahead: {}", LOOKAHEAD_DISTANCE_M, target);
                }
            }

            if let Some(gps_heading) = tracker_lock.get_current_heading() {
                let (gps_dir, _) = heading_to_azimuth_8point(gps_heading);
                println!("  → GPS heading: {:.1}° ({})", gps_heading, gps_dir);
            } else {
                println!("  → GPS heading: N/A (speed too low)");
            }
        }
    }

    Ok(())
}

fn display_status_update(
    tracker: &Arc<Mutex<GpsTracker>>,
    ui: &UserInterface,
    last_status_update: &mut std::time::Instant,
) {
    if last_status_update.elapsed() >= Duration::from_secs(STATUS_UPDATE_INTERVAL_SECS) {
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
                println!("  Speed: {:.2} knots", speed);
            }

            println!();
        }
        *last_status_update = std::time::Instant::now();
    }
}
