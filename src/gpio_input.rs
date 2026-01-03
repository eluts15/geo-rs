use std::error::Error;
use std::thread;
use std::time::Duration;

// Use rppal in production
#[cfg(not(test))]
use rppal::gpio::{Gpio, InputPin, Level};

#[cfg(test)]
// This is only used in testing, not compiled in release.
use crate::mocks::mock_gpio::{Gpio, InputPin, Level};

/// GPIO Pin assignments for 3-way toggle
const GPIO_TOGGLE_LEFT: u8 = 23;
const GPIO_TOGGLE_RIGHT: u8 = 24;

/// Each button press modifies the heading in 5.0 degree increments.
const ADJUST_LEFT_DEGREES: f64 = -5.0;
const ADJUST_RIGHT_DEGREES: f64 = 5.0;

/// Maximum servo deflection angle (degrees)
/// This must match the SERVO_MAX_ANGLE in pwm.rs
const SERVO_MAX_ANGLE: f64 = 90.0;

#[derive(Debug, PartialEq, Clone, Copy)]
/// 3-way toggle positions.
pub enum SwitchPosition {
    Left,
    Neutral,
    Right,
}

pub struct UserInterface {
    toggle_left: InputPin,
    toggle_right: InputPin,
    heading_offset: f64,      // offset from GPS heading (default 0°)
    gps_heading: Option<f64>, // current GPS heading for range limiting
    last_toggle_position: SwitchPosition,
}

/// Provides methods for interacting with GPIO supported physical hardware.
impl UserInterface {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Self::with_pins(GPIO_TOGGLE_LEFT, GPIO_TOGGLE_RIGHT)
    }

    pub fn with_pins(toggle_left_pin: u8, toggle_right_pin: u8) -> Result<Self, Box<dyn Error>> {
        let gpio = Gpio::new()?;

        let toggle_left = gpio.get(toggle_left_pin)?.into_input_pullup();
        let toggle_right = gpio.get(toggle_right_pin)?.into_input_pullup();

        thread::sleep(Duration::from_millis(100));

        Ok(Self {
            toggle_left,
            toggle_right,
            heading_offset: 0.0, // start with no offset (follow GPS)
            gps_heading: None,   // track GPS heading for range limiting
            last_toggle_position: SwitchPosition::Neutral,
        })
    }

    pub fn read_toggle_switch(&self) -> SwitchPosition {
        let left_active = self.toggle_left.read() == Level::Low;
        let right_active = self.toggle_right.read() == Level::Low;

        if left_active && !right_active {
            SwitchPosition::Left
        } else if right_active && !left_active {
            SwitchPosition::Right
        } else {
            SwitchPosition::Neutral
        }
    }

    /// Get the known toggle position.
    pub fn get_toggle_position(&self) -> SwitchPosition {
        self.last_toggle_position
    }

    /// Update the current GPS heading (used for range limiting and calculating target).
    pub fn update_gps_heading(&mut self, heading: f64) {
        self.gps_heading = Some(((heading % 360.0) + 360.0) % 360.0);
    }

    /// Get the target heading (GPS heading + offset).
    /// Returns None if GPS heading not available yet.
    pub fn get_heading(&self) -> Option<f64> {
        self.gps_heading.map(|gps_hdg| {
            let target = gps_hdg + self.heading_offset;
            ((target % 360.0) + 360.0) % 360.0
        })
    }

    /// Get the current heading offset from GPS.
    pub fn get_heading_offset(&self) -> f64 {
        self.heading_offset
    }

    /// Check if GPS heading has been received.
    pub fn has_heading(&self) -> bool {
        self.gps_heading.is_some()
    }

    /// Adjust the heading offset by the specified degrees.
    /// Returns true if adjustment was applied, false if clamped at servo limit.
    fn adjust_heading_offset(&mut self, degrees: f64) -> bool {
        let new_offset = self.heading_offset + degrees;

        // check if new offset exceeds servo limits
        if new_offset.abs() > SERVO_MAX_ANGLE {
            // clamp to servo limit
            self.heading_offset = new_offset.signum() * SERVO_MAX_ANGLE;
            return false; // was clamped
        }

        self.heading_offset = new_offset;
        true // applied successfully
    }

    pub fn update(&mut self) -> Result<bool, Box<dyn Error>> {
        let position = self.read_toggle_switch();
        let position_changed = position != self.last_toggle_position;

        if position_changed {
            self.last_toggle_position = position;

            if self.gps_heading.is_some() {
                match position {
                    SwitchPosition::Left => {
                        let applied = self.adjust_heading_offset(ADJUST_LEFT_DEGREES);
                        if let Some(target_heading) = self.get_heading() {
                            if applied {
                                println!(
                                    "← Toggle LEFT: Offset: {:.1}° | Target: {:.1}°",
                                    self.heading_offset, target_heading
                                );
                            } else {
                                println!(
                                    "← Toggle LEFT: Servo at limit - Offset: {:.1}° | Target: {:.1}°",
                                    self.heading_offset, target_heading
                                );
                                println!("   ⚠ Wait for boat to turn before adjusting further");
                            }
                        }
                    }
                    SwitchPosition::Right => {
                        let applied = self.adjust_heading_offset(ADJUST_RIGHT_DEGREES);
                        if let Some(target_heading) = self.get_heading() {
                            if applied {
                                println!(
                                    "→ Toggle RIGHT: Offset: {:.1}° | Target: {:.1}°",
                                    self.heading_offset, target_heading
                                );
                            } else {
                                println!(
                                    "→ Toggle RIGHT: Servo at limit - Offset: {:.1}° | Target: {:.1}°",
                                    self.heading_offset, target_heading
                                );
                                println!("   ⚠ Wait for boat to turn before adjusting further");
                            }
                        }
                    }
                    SwitchPosition::Neutral => {
                        if let Some(target_heading) = self.get_heading() {
                            println!(
                                "● Toggle NEUTRAL: Offset: {:.1}° | Target: {:.1}°",
                                self.heading_offset, target_heading
                            );
                        }
                    }
                }
            } else {
                println!("⚠ Waiting for GPS heading before adjusting...");
            }
        }

        Ok(position_changed && self.gps_heading.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::mock_gpio;

    #[test]
    fn test_heading_adjustment() {
        let mut heading = 0.0; // start heading NORTH
        heading = ((heading + 5.0) % 360.0 + 360.0) % 360.0; // increment +5.0 degrees
        assert_eq!(heading, 5.0);

        // expect wraparound to work as intended
        // heading = 5.0
        heading = ((heading - 10.0) % 360.0 + 360.0) % 360.0; // degrement -10 degrees
        assert_eq!(heading, 355.0);
    }

    #[test]
    fn test_heading_normalization() {
        let normalize = |h: f64| ((h % 360.0) + 360.0) % 360.0;
        assert_eq!(normalize(370.0), 10.0);
        assert_eq!(normalize(-10.0), 350.0);
    }

    #[test]
    fn test_switch_positions() {
        assert_eq!(SwitchPosition::Left, SwitchPosition::Left);
        assert_ne!(SwitchPosition::Left, SwitchPosition::Right);
    }

    #[test]
    fn test_ui_starts_without_heading() -> Result<(), Box<dyn Error>> {
        mock_gpio::reset_mock_pins();

        let ui = UserInterface::new()?;

        // should start with no GPS heading and zero offset
        assert!(!ui.has_heading());
        assert_eq!(ui.get_heading(), None);
        assert_eq!(ui.get_heading_offset(), 0.0);

        Ok(())
    }

    #[test]
    fn test_toggle_switch_positions() -> Result<(), Box<dyn Error>> {
        mock_gpio::reset_mock_pins();

        let mut ui = UserInterface::new()?;
        // set initial GPS heading
        ui.update_gps_heading(0.0);

        // test neutral
        mock_gpio::set_mock_pin_level(GPIO_TOGGLE_LEFT, mock_gpio::Level::High);
        mock_gpio::set_mock_pin_level(GPIO_TOGGLE_RIGHT, mock_gpio::Level::High);
        ui.update()?;
        assert_eq!(ui.get_toggle_position(), SwitchPosition::Neutral);

        // test LEFT - should create -5° offset
        mock_gpio::set_mock_pin_level(GPIO_TOGGLE_LEFT, mock_gpio::Level::Low);
        thread::sleep(Duration::from_millis(50));
        ui.update()?;
        assert_eq!(ui.get_toggle_position(), SwitchPosition::Left);
        assert_eq!(ui.get_heading_offset(), -5.0);
        assert_eq!(ui.get_heading(), Some(355.0)); // 0° GPS + (-5°) offset

        // back to neutral
        mock_gpio::set_mock_pin_level(GPIO_TOGGLE_LEFT, mock_gpio::Level::High);
        thread::sleep(Duration::from_millis(50));
        ui.update()?;
        assert_eq!(ui.get_toggle_position(), SwitchPosition::Neutral);

        // test RIGHT - should change offset from -5° to 0°
        mock_gpio::set_mock_pin_level(GPIO_TOGGLE_RIGHT, mock_gpio::Level::Low);
        thread::sleep(Duration::from_millis(50));
        ui.update()?;
        assert_eq!(ui.get_toggle_position(), SwitchPosition::Right);
        assert_eq!(ui.get_heading_offset(), 0.0); // -5° + 5° = 0°
        assert_eq!(ui.get_heading(), Some(0.0)); // 0° GPS + 0° offset

        Ok(())
    }

    #[test]
    fn test_heading_wraparound() -> Result<(), Box<dyn Error>> {
        mock_gpio::reset_mock_pins();

        let mut ui = UserInterface::new()?;
        ui.update_gps_heading(358.0);

        // press RIGHT once, offset = +5°, target = 358° + 5° = 363° = 3°
        mock_gpio::set_mock_pin_level(GPIO_TOGGLE_RIGHT, mock_gpio::Level::Low);
        thread::sleep(Duration::from_millis(50));
        ui.update()?;
        assert_eq!(ui.get_heading_offset(), 5.0);
        assert_eq!(ui.get_heading(), Some(3.0));

        Ok(())
    }

    #[test]
    fn test_multiple_adjustments_with_wraparound() -> Result<(), Box<dyn Error>> {
        mock_gpio::reset_mock_pins();

        let mut ui = UserInterface::new()?;
        ui.update_gps_heading(355.0);

        // press RIGHT 3 times: offset = +15°
        // target = 355° + 15° = 370° = 10°
        for _ in 0..3 {
            mock_gpio::set_mock_pin_level(GPIO_TOGGLE_RIGHT, mock_gpio::Level::Low);
            thread::sleep(Duration::from_millis(50));
            ui.update()?;

            mock_gpio::set_mock_pin_level(GPIO_TOGGLE_RIGHT, mock_gpio::Level::High);
            thread::sleep(Duration::from_millis(50));
            ui.update()?;
        }

        assert_eq!(ui.get_heading_offset(), 15.0);
        assert_eq!(ui.get_heading(), Some(10.0));

        Ok(())
    }

    #[test]
    fn test_target_heading_tracks_gps() -> Result<(), Box<dyn Error>> {
        mock_gpio::reset_mock_pins();

        let mut ui = UserInterface::new()?;

        // set initial GPS heading - with no button presses, target should match
        ui.update_gps_heading(45.0);
        assert_eq!(ui.get_heading_offset(), 0.0);
        assert_eq!(ui.get_heading(), Some(45.0));

        // GPS changes to 60° - target should track it
        ui.update_gps_heading(60.0);
        assert_eq!(ui.get_heading_offset(), 0.0);
        assert_eq!(ui.get_heading(), Some(60.0));

        // GPS changes to 180° - target should still track
        ui.update_gps_heading(180.0);
        assert_eq!(ui.get_heading_offset(), 0.0);
        assert_eq!(ui.get_heading(), Some(180.0));

        Ok(())
    }

    #[test]
    fn test_offset_maintains_with_gps_changes() -> Result<(), Box<dyn Error>> {
        mock_gpio::reset_mock_pins();

        let mut ui = UserInterface::new()?;

        // start at GPS 45°
        ui.update_gps_heading(45.0);

        // press RIGHT twice to create +10° offset
        for _ in 0..2 {
            mock_gpio::set_mock_pin_level(GPIO_TOGGLE_RIGHT, mock_gpio::Level::Low);
            thread::sleep(Duration::from_millis(50));
            ui.update()?;
            mock_gpio::set_mock_pin_level(GPIO_TOGGLE_RIGHT, mock_gpio::Level::High);
            thread::sleep(Duration::from_millis(50));
            ui.update()?;
        }

        assert_eq!(ui.get_heading_offset(), 10.0);
        assert_eq!(ui.get_heading(), Some(55.0)); // 45° + 10°

        // GPS changes to 60° as boat turns - offset should maintain
        ui.update_gps_heading(60.0);
        assert_eq!(ui.get_heading_offset(), 10.0); // Offset unchanged
        assert_eq!(ui.get_heading(), Some(70.0)); // 60° + 10°

        // GPS changes to 100° - offset still maintains
        ui.update_gps_heading(100.0);
        assert_eq!(ui.get_heading_offset(), 10.0); // Offset unchanged
        assert_eq!(ui.get_heading(), Some(110.0)); // 100° + 10°

        Ok(())
    }

    #[test]
    fn test_servo_range_limiting() -> Result<(), Box<dyn Error>> {
        mock_gpio::reset_mock_pins();

        let mut ui = UserInterface::new()?;

        // set GPS heading to 0° (North)
        ui.update_gps_heading(0.0);

        // try to create +100° offset (beyond +90° servo limit)
        for _ in 0..20 {
            // 20 * 5° = 100°
            mock_gpio::set_mock_pin_level(GPIO_TOGGLE_RIGHT, mock_gpio::Level::Low);
            thread::sleep(Duration::from_millis(50));
            ui.update()?;

            mock_gpio::set_mock_pin_level(GPIO_TOGGLE_RIGHT, mock_gpio::Level::High);
            thread::sleep(Duration::from_millis(50));
            ui.update()?;
        }

        // offset should be clamped to +90°
        assert_eq!(ui.get_heading_offset(), 90.0);
        // target heading should be 0° + 90° = 90°
        let heading = ui.get_heading().unwrap();
        assert!(
            (heading - 90.0).abs() < 1.0,
            "Heading should be clamped to ~90°, got {:.1}°",
            heading
        );

        Ok(())
    }

    #[test]
    fn test_servo_range_limiting_left() -> Result<(), Box<dyn Error>> {
        mock_gpio::reset_mock_pins();

        let mut ui = UserInterface::new()?;

        // set GPS heading to 180° (South)
        ui.update_gps_heading(180.0);

        // try to create -100° offset (beyond -90° servo limit)
        for _ in 0..20 {
            // 20 * -5° = -100°
            mock_gpio::set_mock_pin_level(GPIO_TOGGLE_LEFT, mock_gpio::Level::Low);
            thread::sleep(Duration::from_millis(50));
            ui.update()?;

            mock_gpio::set_mock_pin_level(GPIO_TOGGLE_LEFT, mock_gpio::Level::High);
            thread::sleep(Duration::from_millis(50));
            ui.update()?;
        }

        // offset should be clamped to -90°
        assert_eq!(ui.get_heading_offset(), -90.0);
        // target heading should be 180° - 90° = 90°
        let heading = ui.get_heading().unwrap();
        assert!(
            (heading - 90.0).abs() < 1.0,
            "Heading should be clamped to ~90°, got {:.1}°",
            heading
        );

        Ok(())
    }

    #[test]
    fn test_servo_range_with_wraparound() -> Result<(), Box<dyn Error>> {
        mock_gpio::reset_mock_pins();

        let mut ui = UserInterface::new()?;

        // set GPS heading to 10° (just past North)
        ui.update_gps_heading(10.0);

        // try to create -100° offset (beyond -90° limit)
        for _ in 0..20 {
            // 20 * -5° = -100°
            mock_gpio::set_mock_pin_level(GPIO_TOGGLE_LEFT, mock_gpio::Level::Low);
            thread::sleep(Duration::from_millis(50));
            ui.update()?;

            mock_gpio::set_mock_pin_level(GPIO_TOGGLE_LEFT, mock_gpio::Level::High);
            thread::sleep(Duration::from_millis(50));
            ui.update()?;
        }

        // offset should be clamped to -90°
        assert_eq!(ui.get_heading_offset(), -90.0);
        // target heading: 10° - 90° = -80° → 280°
        let heading = ui.get_heading().unwrap();
        assert!(
            (heading - 280.0).abs() < 1.0,
            "Heading should be clamped to ~280°, got {:.1}°",
            heading
        );

        Ok(())
    }
}
