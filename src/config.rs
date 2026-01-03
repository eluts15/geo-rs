// ** CALIBRATION CONFIGURATION ** //

// Magnetometer calibration offsets (hard iron correction)
// Obtained from calibration: rotate board 360° and record min/max X,Y values
pub const X_OFFSET: f64 = -2776.0; // (X_min + X_max) / 2
pub const Y_OFFSET: f64 = -72776.0; // (Y_min + Y_max) / 2
pub const HEADING_OFFSET: f64 = 88.0; // Overall heading correction for this location

// ** GPIO CONFIGURATION ** //
/// GPIO Pin assignments for 3-way toggle
pub const GPIO_TOGGLE_LEFT: u8 = 23;
pub const GPIO_TOGGLE_RIGHT: u8 = 24;
/// Each button press modifies the heading in 5.0 degree increments.
pub const ADJUST_LEFT_DEGREES: f64 = -5.0;
pub const ADJUST_RIGHT_DEGREES: f64 = 5.0;

//  ** SERVO CONFIGURATION ** //

/// GPIO PWM channel for servo control
/// - GPIO 18 (PWM0) - Most commonly used
pub const SERVO_PWM_PIN: u8 = 18;
/// Standard servo pulse width range (microseconds)
/// Most servos use 1000-2000μs, with 1500μs as center
pub const SERVO_MIN_PULSE_US: f64 = 1000.0;
pub const SERVO_MAX_PULSE_US: f64 = 2000.0;
pub const SERVO_CENTER_PULSE_US: f64 = 1500.0;
/// Standard servo PWM frequency (Hz)
pub const SERVO_FREQUENCY_HZ: f64 = 50.0;
/// Maximum servo angle range (degrees)
/// Typical servos have 180° or 90° range
pub const SERVO_MAX_ANGLE: f64 = 90.0;
/// PID controller gains
pub const KP: f64 = 1.0; // Proportional gain
pub const KI: f64 = 0.0; // Integral gain (disabled for now)
pub const KD: f64 = 0.0; // Derivative gain (disabled for now)
/// Maximum heading error before applying correction (degrees)
pub const HEADING_ERROR_DEADBAND: f64 = 2.0;
/// Maximum servo movement rate (degrees per second)
/// This prevents violent rudder movements that could destabilize the boat
pub const MAX_SERVO_RATE: f64 = 40.0;
pub const SERVO_UPDATE_INTERVAL_SECS: f64 = 0.1; // 10Hz

// ** MAIN CONFIGURATION ** //
pub const LOOKAHEAD_DISTANCE_M: f64 = 100.0;
pub const STATUS_UPDATE_INTERVAL_SECS: u64 = 1;
