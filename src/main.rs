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
use geo_rs::compass::heading_to_azimuth_8point;
use geo_rs::pwm::ServoController;
use gpio_input::UserInterface;

const LOOKAHEAD_DISTANCE_M: f64 = 100.0;
const STATUS_UPDATE_INTERVAL_SECS: u64 = 5;
const SERVO_UPDATE_INTERVAL_SECS: f64 = 0.1; // 10Hz

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting GPS Navigation System...");

    let tracker = Arc::new(Mutex::new(GpsTracker::new()));
    let mut ui = UserInterface::new()?;

    // Try to initialize servo, but don't fail if it's unavailable
    let mut servo = match ServoController::new() {
        Ok(s) => {
            println!("Servo controller initialized on GPIO 18");
            Some(s)
        }
        Err(e) => {
            eprintln!("Servo not available: {} - running in manual mode only", e);
            None
        }
    };

    initialize_system(&mut servo)?;
    start_gps_thread(Arc::clone(&tracker));
    wait_for_gps_fix(&tracker, &mut ui)?;

    run_main_loop(&tracker, &mut ui, &mut servo)?;

    Ok(())
}

fn initialize_system(
    servo: &mut Option<ServoController>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("GPIO initialized:");
    println!("  Toggle Left:  GPIO 23");
    println!("  Toggle Right: GPIO 24");
    println!("  Servo PWM:    GPIO 18");

    servo.iter();
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
            drop(tracker_lock);

            if let Some(sats) = num_sats {
                println!("  Satellites: {}", sats);
            }

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
    servo: &mut Option<ServoController>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut last_status_update = std::time::Instant::now();
    let mut last_servo_update = std::time::Instant::now();
    let mut last_correction = 0.0; // Track last correction to reduce noise

    loop {
        initialize_heading_if_needed(tracker, ui);
        handle_toggle_changes(tracker, ui)?;

        // Only apply servo correction if servo is available
        if let Some(s) = servo {
            apply_servo_correction(tracker, ui, s, &mut last_servo_update, &mut last_correction)?;
        }

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
                    tracker_lock.get_vector_to_azimuth(target_heading, LOOKAHEAD_DISTANCE_M)
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

fn apply_servo_correction(
    tracker: &Arc<Mutex<GpsTracker>>,
    ui: &UserInterface,
    servo: &mut ServoController,
    last_servo_update: &mut std::time::Instant,
    last_correction: &mut f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let dt = last_servo_update.elapsed().as_secs_f64();

    if dt >= SERVO_UPDATE_INTERVAL_SECS {
        if let Some(target_heading) = ui.get_heading() {
            if let Ok(tracker_lock) = tracker.lock() {
                if let Some(gps_heading) = tracker_lock.get_current_heading() {
                    match servo.auto_steer(target_heading, gps_heading, dt) {
                        Ok(correction) => {
                            // Only print if correction changed by more than 0.5°
                            let correction_change = (correction - *last_correction).abs();
                            if correction_change > 0.5 {
                                if correction.abs() > 0.1 {
                                    println!("  ⚙ Steering correction: {:.1}°", correction);
                                }
                                *last_correction = correction;
                            }
                        }
                        Err(e) => eprintln!("Servo error: {}", e),
                    }
                }
            }
        }
        *last_servo_update = std::time::Instant::now();
    }

    Ok(())
}
