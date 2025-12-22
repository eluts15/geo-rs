use rppal::gpio::{Gpio, InputPin, Level};
use std::error::Error;
use std::thread;
use std::time::Duration;

// GPIO Pin assignments for 3-way toggle
const GPIO_TOGGLE_LEFT: u8 = 23; // 3-way toggle - left position
const GPIO_TOGGLE_RIGHT: u8 = 24; // 3-way toggle - right position

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SwitchPosition {
    Left,
    Neutral,
    Right,
}

pub struct UserInterface {
    // GPIO pins for 3-way toggle
    toggle_left: InputPin,
    toggle_right: InputPin,

    // System state
    current_heading: f64, // degrees - this is our target/desired heading
    last_toggle_position: SwitchPosition,
}

impl UserInterface {
    /// Create a new UserInterface with default GPIO pins
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Self::with_pins(GPIO_TOGGLE_LEFT, GPIO_TOGGLE_RIGHT)
    }

    /// Create a new UserInterface with custom GPIO pins
    ///
    /// # Arguments
    /// * `toggle_left_pin` - GPIO pin for toggle left position
    /// * `toggle_right_pin` - GPIO pin for toggle right position
    pub fn with_pins(toggle_left_pin: u8, toggle_right_pin: u8) -> Result<Self, Box<dyn Error>> {
        let gpio = Gpio::new()?;

        // Set up pins with pull-up resistors (active low)
        let toggle_left = gpio.get(toggle_left_pin)?.into_input_pullup();
        let toggle_right = gpio.get(toggle_right_pin)?.into_input_pullup();

        // Wait for pins to stabilize
        thread::sleep(Duration::from_millis(100));

        Ok(Self {
            toggle_left,
            toggle_right,
            current_heading: 0.0,
            last_toggle_position: SwitchPosition::Neutral,
        })
    }

    /// Read the 3-way toggle switch position (active low)
    pub fn read_toggle_switch(&self) -> SwitchPosition {
        let left_active = self.toggle_left.read() == Level::Low;
        let right_active = self.toggle_right.read() == Level::Low;

        if left_active && !right_active {
            SwitchPosition::Left
        } else if right_active && !left_active {
            SwitchPosition::Right
        } else {
            // Neither pressed or both pressed = Neutral
            SwitchPosition::Neutral
        }
    }

    /// Set the current heading (typically called when syncing with GPS)
    pub fn set_heading(&mut self, heading: f64) {
        // Normalize heading to 0-360 range
        self.current_heading = ((heading % 360.0) + 360.0) % 360.0;
    }

    /// Get the current target heading
    pub fn get_heading(&self) -> f64 {
        self.current_heading
    }

    /// Adjust heading by a given amount (positive = right, negative = left)
    fn adjust_heading(&mut self, degrees: f64) {
        self.current_heading = ((self.current_heading + degrees) % 360.0 + 360.0) % 360.0;
    }

    /// Main control loop - checks switch and adjusts heading
    ///
    /// When left is pressed: adjusts heading -5° (turn left)
    /// When right is pressed: adjusts heading +5° (turn right)
    /// When neutral: maintains current heading
    ///
    /// Returns true if toggle position changed
    pub fn update(&mut self) -> Result<bool, Box<dyn Error>> {
        // Read the toggle switch position
        let position = self.read_toggle_switch();

        // Check if position changed
        let position_changed = position != self.last_toggle_position;

        if position_changed {
            self.last_toggle_position = position;

            match position {
                SwitchPosition::Left => {
                    self.adjust_heading(-5.0);
                    println!("← Toggle LEFT: New heading: {:.1}°", self.current_heading);
                }
                SwitchPosition::Right => {
                    self.adjust_heading(5.0);
                    println!("→ Toggle RIGHT: New heading: {:.1}°", self.current_heading);
                }
                SwitchPosition::Neutral => {
                    println!(
                        "● Toggle NEUTRAL: Current heading: {:.1}°",
                        self.current_heading
                    );
                }
            }
        }

        Ok(position_changed)
    }

    /// Get the last toggle position without updating state
    pub fn get_toggle_position(&self) -> SwitchPosition {
        self.last_toggle_position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_adjustment() {
        // Mock test without actual GPIO
        let mut heading = 0.0;

        // Test right adjustment (+5°)
        heading = ((heading + 5.0) % 360.0 + 360.0) % 360.0;
        assert_eq!(heading, 5.0);

        // Test left adjustment (-10°)
        heading = ((heading - 10.0) % 360.0 + 360.0) % 360.0;
        assert_eq!(heading, 355.0);

        // Test wraparound
        heading = ((heading + 10.0) % 360.0 + 360.0) % 360.0;
        assert_eq!(heading, 5.0);
    }

    #[test]
    fn test_heading_normalization() {
        let normalize = |h: f64| ((h % 360.0) + 360.0) % 360.0;

        assert_eq!(normalize(370.0), 10.0);
        assert_eq!(normalize(-10.0), 350.0);
        assert_eq!(normalize(720.0), 0.0);
    }

    #[test]
    fn test_switch_positions() {
        assert_eq!(SwitchPosition::Left, SwitchPosition::Left);
        assert_eq!(SwitchPosition::Right, SwitchPosition::Right);
        assert_eq!(SwitchPosition::Neutral, SwitchPosition::Neutral);
        assert_ne!(SwitchPosition::Left, SwitchPosition::Right);
    }
}
