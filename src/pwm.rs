use std::error::Error;

// Use rppal in production
#[cfg(not(test))]
use rppal::pwm::{Channel, Polarity, Pwm};

// Mock PWM for testing
#[cfg(test)]
use crate::mock_pwm::Pwm;

/// GPIO PWM channel for servo control
/// Hardware PWM is available on:
/// - GPIO 12 (PWM0)
/// - GPIO 13 (PWM1)
/// - GPIO 18 (PWM0) - Most commonly used
/// - GPIO 19 (PWM1)
const SERVO_PWM_PIN: u8 = 18;

/// Standard servo pulse width range (microseconds)
/// Most servos use 1000-2000μs, with 1500μs as center
const SERVO_MIN_PULSE_US: f64 = 1000.0;
const SERVO_MAX_PULSE_US: f64 = 2000.0;
const SERVO_CENTER_PULSE_US: f64 = 1500.0;

/// Standard servo PWM frequency (Hz)
const SERVO_FREQUENCY_HZ: f64 = 50.0;

/// Maximum servo angle range (degrees)
/// Typical servos have 180° or 90° range
const SERVO_MAX_ANGLE: f64 = 90.0;

/// PID controller gains
const KP: f64 = 1.0; // Proportional gain
const KI: f64 = 0.0; // Integral gain (disabled for now)
const KD: f64 = 0.0; // Derivative gain (disabled for now)

/// Maximum heading error before applying correction (degrees)
const HEADING_ERROR_DEADBAND: f64 = 2.0;

pub struct ServoController {
    pwm: Pwm,
    integral: f64,
    last_error: f64,
}

impl ServoController {
    /// Create a new servo controller
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Self::with_pin(SERVO_PWM_PIN)
    }

    /// Create a servo controller with a custom GPIO pin
    pub fn with_pin(pin: u8) -> Result<Self, Box<dyn Error>> {
        #[cfg(not(test))]
        let channel = match pin {
            12 | 18 => Channel::Pwm0,
            13 | 19 => Channel::Pwm1,
            _ => return Err("Invalid PWM pin. Use 12, 13, 18, or 19".into()),
        };

        #[cfg(not(test))]
        let pwm = Pwm::with_frequency(
            channel,
            SERVO_FREQUENCY_HZ,
            0.5, // 50% duty cycle (centered)
            Polarity::Normal,
            true, // enabled
        )?;

        #[cfg(test)]
        let pwm = Pwm::new(pin)?;

        Ok(Self {
            pwm,
            integral: 0.0,
            last_error: 0.0,
        })
    }

    /// Set servo position based on angle (-SERVO_MAX_ANGLE to +SERVO_MAX_ANGLE)
    /// Negative = left, Positive = right, 0 = center
    pub fn set_angle(&mut self, angle: f64) -> Result<(), Box<dyn Error>> {
        // Clamp angle to valid range
        let clamped_angle = angle.clamp(-SERVO_MAX_ANGLE, SERVO_MAX_ANGLE);

        // Map angle to pulse width
        // -90° → 1000μs, 0° → 1500μs, +90° → 2000μs
        let pulse_us = SERVO_CENTER_PULSE_US
            + (clamped_angle / SERVO_MAX_ANGLE) * (SERVO_MAX_PULSE_US - SERVO_CENTER_PULSE_US);

        self.set_pulse_width_us(pulse_us)
    }

    /// Set servo to center position (neutral)
    pub fn center(&mut self) -> Result<(), Box<dyn Error>> {
        self.set_angle(0.0)
    }

    /// Set servo pulse width directly (microseconds)
    fn set_pulse_width_us(&mut self, pulse_us: f64) -> Result<(), Box<dyn Error>> {
        // Clamp to valid range
        let clamped_pulse = pulse_us.clamp(SERVO_MIN_PULSE_US, SERVO_MAX_PULSE_US);

        // Convert to duty cycle
        // Period = 1/50Hz = 20ms = 20000μs
        let period_us = 1_000_000.0 / SERVO_FREQUENCY_HZ;
        let duty_cycle = clamped_pulse / period_us;

        #[cfg(not(test))]
        self.pwm.set_duty_cycle(duty_cycle)?;

        #[cfg(test)]
        self.pwm.set_duty_cycle(duty_cycle);

        Ok(())
    }

    /// Calculate steering correction using PID controller
    ///
    /// # Arguments
    /// * `target_heading` - Desired heading (degrees)
    /// * `current_heading` - Actual GPS heading (degrees)
    /// * `dt` - Time delta since last update (seconds)
    ///
    /// # Returns
    /// Servo angle correction (-SERVO_MAX_ANGLE to +SERVO_MAX_ANGLE)
    pub fn calculate_correction(
        &mut self,
        target_heading: f64,
        current_heading: f64,
        dt: f64,
    ) -> f64 {
        // Calculate heading error (accounting for wraparound)
        let mut error = target_heading - current_heading;

        // Normalize error to -180 to +180 range
        if error > 180.0 {
            error -= 360.0;
        } else if error < -180.0 {
            error += 360.0;
        }

        // Apply deadband - don't correct small errors
        if error.abs() < HEADING_ERROR_DEADBAND {
            return 0.0;
        }

        // PID calculations
        let p_term = KP * error;

        self.integral += error * dt;
        let i_term = KI * self.integral;

        let derivative = (error - self.last_error) / dt;
        let d_term = KD * derivative;

        self.last_error = error;

        // Calculate correction angle
        let correction = p_term + i_term + d_term;

        // Clamp to servo limits
        correction.clamp(-SERVO_MAX_ANGLE, SERVO_MAX_ANGLE)
    }

    /// Apply automatic heading correction
    ///
    /// # Arguments
    /// * `target_heading` - Desired heading (degrees)
    /// * `current_heading` - Actual GPS heading (degrees)
    /// * `dt` - Time delta since last update (seconds)
    pub fn auto_steer(
        &mut self,
        target_heading: f64,
        current_heading: f64,
        dt: f64,
    ) -> Result<f64, Box<dyn Error>> {
        let correction = self.calculate_correction(target_heading, current_heading, dt);
        self.set_angle(correction)?;
        Ok(correction)
    }

    /// Disable PWM output
    pub fn disable(&mut self) -> Result<(), Box<dyn Error>> {
        #[cfg(not(test))]
        self.pwm.disable()?;

        Ok(())
    }
}

impl Drop for ServoController {
    fn drop(&mut self) {
        // Ensure PWM is disabled when dropped
        let _ = self.disable();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_error_calculation() {
        let mut controller = ServoController::new().unwrap();

        // Test simple error
        let correction = controller.calculate_correction(90.0, 85.0, 0.1);
        assert!(correction > 0.0); // Should correct right

        // Test wraparound (target 5°, current 355°)
        let correction = controller.calculate_correction(5.0, 355.0, 0.1);
        assert!(correction > 0.0); // Should correct right

        // Test wraparound (target 355°, current 5°)
        let correction = controller.calculate_correction(355.0, 5.0, 0.1);
        assert!(correction < 0.0); // Should correct left
    }

    #[test]
    fn test_deadband() {
        let mut controller = ServoController::new().unwrap();

        // Small error within deadband - should return 0
        let correction = controller.calculate_correction(90.0, 89.0, 0.1);
        assert_eq!(correction, 0.0);

        // Large error outside deadband - should return correction
        let correction = controller.calculate_correction(90.0, 80.0, 0.1);
        assert!(correction != 0.0);
    }

    #[test]
    fn test_servo_angle_clamping() {
        let mut controller = ServoController::new().unwrap();

        // Test angle clamping
        assert!(controller.set_angle(100.0).is_ok()); // Should clamp to max
        assert!(controller.set_angle(-100.0).is_ok()); // Should clamp to min
        assert!(controller.set_angle(0.0).is_ok()); // Center
    }
}
