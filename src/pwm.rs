use std::error::Error;

// Use rppal in production
#[cfg(not(test))]
use rppal::pwm::{Channel, Polarity, Pwm};

// Mock PWM for testing
use crate::config::{
    HEADING_ERROR_DEADBAND, KD, KI, KP, MAX_SERVO_RATE, SERVO_CENTER_PULSE_US, SERVO_FREQUENCY_HZ,
    SERVO_MAX_ANGLE, SERVO_MAX_PULSE_US, SERVO_MIN_PULSE_US, SERVO_PWM_PIN,
};

#[cfg(test)]
use crate::mocks::mock_pwm::Pwm;

pub struct ServoController {
    pwm: Pwm,
    integral: f64,
    last_error: f64,
    current_angle: f64, // track current servo position for rate limiting
}

impl ServoController {
    /// Create a new servo controller.
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Self::with_pin(SERVO_PWM_PIN)
    }

    /// Create a servo controller with a custom GPIO pin.
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
            current_angle: 0.0, // start at center position
        })
    }

    /// Set servo position based on angle (-SERVO_MAX_ANGLE to +SERVO_MAX_ANGLE)
    /// Negative = left, Positive = right, 0 = center
    pub fn set_angle(&mut self, angle: f64) -> Result<(), Box<dyn Error>> {
        // clamp angle to valid range
        let clamped_angle = angle.clamp(-SERVO_MAX_ANGLE, SERVO_MAX_ANGLE);

        // map angle to pulse width
        // -90° → 1000μs, 0° → 1500μs, +90° → 2000μs
        let pulse_us = SERVO_CENTER_PULSE_US
            + (clamped_angle / SERVO_MAX_ANGLE) * (SERVO_MAX_PULSE_US - SERVO_CENTER_PULSE_US);

        self.set_pulse_width_us(pulse_us)?;

        // track current position
        self.current_angle = clamped_angle;

        Ok(())
    }

    /// Set servo to center position (neutral).
    pub fn center(&mut self) -> Result<(), Box<dyn Error>> {
        self.set_angle(0.0)
    }

    /// Set servo pulse width directly (microseconds).
    fn set_pulse_width_us(&mut self, pulse_us: f64) -> Result<(), Box<dyn Error>> {
        // clamp to valid range
        let clamped_pulse = pulse_us.clamp(SERVO_MIN_PULSE_US, SERVO_MAX_PULSE_US);

        // convert to duty cycle
        // period = 1/50Hz = 20ms = 20000μs
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
        // calculate heading error (accounting for wraparound)
        // NOTE: For boat rudder control, the error sign is inverted.
        // when heading is too far right, we need positive rudder angle (rudder right)
        // to push the stern right and turn the bow left.
        let mut error = current_heading - target_heading;

        // normalize error to -180 to +180 range
        if error > 180.0 {
            error -= 360.0;
        } else if error < -180.0 {
            error += 360.0;
        }

        // apply deadband - don't correct small errors
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

        // calculate correction angle
        let correction = p_term + i_term + d_term;

        // clamp to servo limits
        correction.clamp(-SERVO_MAX_ANGLE, SERVO_MAX_ANGLE)
    }

    /// Apply automatic heading correction with rate limiting
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
        let desired_correction = self.calculate_correction(target_heading, current_heading, dt);

        // apply rate limiting to prevent violent movements
        let max_change = MAX_SERVO_RATE * dt;
        let angle_diff = desired_correction - self.current_angle;

        let actual_correction = if angle_diff.abs() > max_change {
            // limit the change to maximum rate
            self.current_angle + angle_diff.signum() * max_change
        } else {
            // small change, apply directly
            desired_correction
        };

        self.set_angle(actual_correction)?;
        Ok(actual_correction)
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
        let _ = self.disable();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_error_calculation() {
        let mut controller = ServoController::new().unwrap();

        // test simple error: current heading is 85° (too far left), target is 90°
        // rudder should move LEFT (negative) to turn bow right toward target
        let correction = controller.calculate_correction(90.0, 85.0, 0.1);
        assert!(correction < 0.0); // should correct left (negative rudder angle)

        // test wraparound: target 5°, current 355° (too far left of target)
        // rudder should move LEFT (negative) to turn bow right toward 5°
        let correction = controller.calculate_correction(5.0, 355.0, 0.1);
        assert!(correction < 0.0); // should correct left

        // test wraparound: target 355°, current 5° (too far right of target)
        // rudder should move RIGHT (positive) to turn bow left toward 355°
        let correction = controller.calculate_correction(355.0, 5.0, 0.1);
        assert!(correction > 0.0); // should correct right
    }

    #[test]
    fn test_deadband() {
        let mut controller = ServoController::new().unwrap();

        // small error within deadband - should return 0
        let correction = controller.calculate_correction(90.0, 89.0, 0.1);
        assert_eq!(correction, 0.0);

        // large error outside deadband - should return correction
        let correction = controller.calculate_correction(90.0, 80.0, 0.1);
        assert!(correction != 0.0);
    }

    #[test]
    fn test_servo_angle_clamping() {
        let mut controller = ServoController::new().unwrap();

        // test angle clamping
        assert!(controller.set_angle(100.0).is_ok()); // should clamp to max
        assert!(controller.set_angle(-100.0).is_ok()); // should clamp to min
        assert!(controller.set_angle(0.0).is_ok()); // center
    }

    #[test]
    fn test_boat_rudder_steering_logic() {
        let mut controller = ServoController::new().unwrap();

        // Scenario 1: Boat heading too far RIGHT (100°), need to go back to 90°
        // Error = 100° - 90° = +10° (positive error)
        // Correction should be POSITIVE (rudder moves right)
        // This pushes stern right, turning bow LEFT back toward 90°
        let correction = controller.calculate_correction(90.0, 100.0, 0.1);
        assert!(
            correction > 0.0,
            "Rudder should move RIGHT when heading too far right"
        );

        // Scenario 2: Boat heading too far LEFT (80°), need to go to 90°
        // Error = 80° - 90° = -10° (negative error)
        // Correction should be NEGATIVE (rudder moves left)
        // This pushes stern left, turning bow RIGHT toward 90°
        let correction = controller.calculate_correction(90.0, 80.0, 0.1);
        assert!(
            correction < 0.0,
            "Rudder should move LEFT when heading too far left"
        );

        // Scenario 3: On target - should return 0 within deadband
        let correction = controller.calculate_correction(90.0, 90.5, 0.1);
        assert_eq!(correction, 0.0, "Should not correct when within deadband");
    }

    #[test]
    fn test_servo_rate_limiting() {
        let mut controller = ServoController::new().unwrap();

        // start at center (0°)
        assert_eq!(controller.current_angle, 0.0);

        // large correction needed: heading 0°, target 90° → error = -90°
        // with dt=0.1s and MAX_SERVO_RATE=20°/s, max change = 2°
        let correction = controller.auto_steer(90.0, 0.0, 0.1).unwrap();

        // should move toward -90° but limited to 2° change
        assert!(
            (correction - (-2.0)).abs() < 0.1,
            "First step should be limited to -2°, got {:.1}°",
            correction
        );
        assert_eq!(controller.current_angle, correction);

        // next update: should move another 2°
        let correction = controller.auto_steer(90.0, 0.0, 0.1).unwrap();
        assert!(
            (correction - (-4.0)).abs() < 0.1,
            "Second step should be -4° total, got {:.1}°",
            correction
        );

        // simulate reaching close to target
        controller.current_angle = -88.0;

        // small correction needed, should apply directly (no rate limit needed)
        let correction = controller.auto_steer(90.0, 2.0, 0.1).unwrap();
        assert!(
            (correction - (-88.0)).abs() < 0.1,
            "Small corrections should apply directly"
        );
    }
}
