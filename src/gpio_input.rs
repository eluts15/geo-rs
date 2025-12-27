use std::error::Error;
use std::thread;
use std::time::Duration;

// Use rppal in production
#[cfg(not(test))]
use rppal::gpio::{Gpio, InputPin, Level};

#[cfg(test)]
// This is only used in testing, not compiled in release.
use crate::mock_gpio::{Gpio, InputPin, Level};

/// GPIO Pin assignments for 3-way toggle
const GPIO_TOGGLE_LEFT: u8 = 23;
const GPIO_TOGGLE_RIGHT: u8 = 24;

/// Each button press modifies the heading in 5.0 degree increments.
const ADJUST_LEFT_DEGREES: f64 = -5.0;
const ADJUST_RIGHT_DEGREES: f64 = 5.0;

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
    current_heading: Option<f64>, // None until GPS provides actual heading
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
            current_heading: None, // Wait for GPS to provide actual heading
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

    /// Set the heading from GPS.
    pub fn set_heading(&mut self, heading: f64) {
        self.current_heading = Some(((heading % 360.0) + 360.0) % 360.0);
    }

    /// Get the current heading. Returns None if GPS hasn't initialized it yet.
    pub fn get_heading(&self) -> Option<f64> {
        self.current_heading
    }

    /// Check if heading has been initialized from GPS.
    pub fn has_heading(&self) -> bool {
        self.current_heading.is_some()
    }

    /// Set a new heading by adjusting current heading.
    fn adjust_heading(&mut self, degrees: f64) {
        if let Some(heading) = self.current_heading {
            self.current_heading = Some(((heading + degrees) % 360.0 + 360.0) % 360.0);
        }
    }

    pub fn update(&mut self) -> Result<bool, Box<dyn Error>> {
        let position = self.read_toggle_switch();
        let position_changed = position != self.last_toggle_position;

        if position_changed {
            self.last_toggle_position = position;

            if self.current_heading.is_some() {
                match position {
                    SwitchPosition::Left => {
                        self.adjust_heading(ADJUST_LEFT_DEGREES);
                        if let Some(heading) = self.current_heading {
                            println!("← Toggle LEFT: New heading: {:.1}°", heading);
                        }
                    }
                    SwitchPosition::Right => {
                        self.adjust_heading(ADJUST_RIGHT_DEGREES);
                        if let Some(heading) = self.current_heading {
                            println!("→ Toggle RIGHT: New heading: {:.1}°", heading);
                        }
                    }
                    SwitchPosition::Neutral => {
                        if let Some(heading) = self.current_heading {
                            println!("● Toggle NEUTRAL: Current heading: {:.1}°", heading);
                        }
                    }
                }
            } else {
                println!("⚠ Waiting for GPS heading before adjusting...");
            }
        }

        Ok(position_changed && self.current_heading.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_gpio;

    // region: UNIT_TESTS
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

        // Should start with no heading until GPS provides it
        assert!(!ui.has_heading());
        assert_eq!(ui.get_heading(), None);

        Ok(())
    }
    // endregion: UNIT_TESTS

    // region MOCK: Mocking GPIO Functionality.
    #[test]
    fn test_toggle_switch_positions() -> Result<(), Box<dyn Error>> {
        mock_gpio::reset_mock_pins();

        let mut ui = UserInterface::new()?;
        // Set initial heading like GPS would
        ui.set_heading(0.0);

        // Test neutral
        mock_gpio::set_mock_pin_level(GPIO_TOGGLE_LEFT, mock_gpio::Level::High);
        mock_gpio::set_mock_pin_level(GPIO_TOGGLE_RIGHT, mock_gpio::Level::High);
        ui.update()?;
        assert_eq!(ui.get_toggle_position(), SwitchPosition::Neutral);

        // Test LEFT
        mock_gpio::set_mock_pin_level(GPIO_TOGGLE_LEFT, mock_gpio::Level::Low);
        thread::sleep(Duration::from_millis(50));
        ui.update()?;
        assert_eq!(ui.get_toggle_position(), SwitchPosition::Left);
        assert_eq!(ui.get_heading(), Some(355.0));

        // Back to neutral
        mock_gpio::set_mock_pin_level(GPIO_TOGGLE_LEFT, mock_gpio::Level::High);
        thread::sleep(Duration::from_millis(50));
        ui.update()?;
        assert_eq!(ui.get_toggle_position(), SwitchPosition::Neutral);

        // Test RIGHT
        mock_gpio::set_mock_pin_level(GPIO_TOGGLE_RIGHT, mock_gpio::Level::Low);
        thread::sleep(Duration::from_millis(50));
        ui.update()?;
        assert_eq!(ui.get_toggle_position(), SwitchPosition::Right);
        assert_eq!(ui.get_heading(), Some(0.0));

        Ok(())
    }

    #[test]
    fn test_heading_wraparound() -> Result<(), Box<dyn Error>> {
        mock_gpio::reset_mock_pins();

        let mut ui = UserInterface::new()?;
        ui.set_heading(358.0);

        // Press RIGHT once, expect it to wraparound to 3.0
        mock_gpio::set_mock_pin_level(GPIO_TOGGLE_RIGHT, mock_gpio::Level::Low);
        thread::sleep(Duration::from_millis(50));
        ui.update()?;
        assert_eq!(ui.get_heading(), Some(3.0));

        Ok(())
    }

    #[test]
    fn test_multiple_adjustments_with_wraparound() -> Result<(), Box<dyn Error>> {
        mock_gpio::reset_mock_pins();

        let mut ui = UserInterface::new()?;
        ui.set_heading(355.0);

        // 355.0 + 5.0 = 0.0
        // 0.0 + 5.0 = 5.0
        // 5.0 + 5.0 = 10.0
        for _ in 0..3 {
            mock_gpio::set_mock_pin_level(GPIO_TOGGLE_RIGHT, mock_gpio::Level::Low);
            thread::sleep(Duration::from_millis(50));
            ui.update()?;

            mock_gpio::set_mock_pin_level(GPIO_TOGGLE_RIGHT, mock_gpio::Level::High);
            thread::sleep(Duration::from_millis(50));
            ui.update()?;
        }

        assert_eq!(ui.get_heading(), Some(10.0));

        Ok(())
    }
    // endregion: MOCK: Mocking GPIO Functionality.
}
