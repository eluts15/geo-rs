use std::error::Error;
use std::thread;
use std::time::Duration;

// Use real rppal in production
#[cfg(not(test))]
use rppal::gpio::{Gpio, InputPin, Level};

#[cfg(test)]
use crate::mock_gpio::{Gpio, InputPin, Level};

// GPIO Pin assignments for 3-way toggle
const GPIO_TOGGLE_LEFT: u8 = 23;
const GPIO_TOGGLE_RIGHT: u8 = 24;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SwitchPosition {
    Left,
    Neutral,
    Right,
}

pub struct UserInterface {
    toggle_left: InputPin,
    toggle_right: InputPin,
    current_heading: f64,
    last_toggle_position: SwitchPosition,
}

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
            current_heading: 0.0,
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

    pub fn set_heading(&mut self, heading: f64) {
        self.current_heading = ((heading % 360.0) + 360.0) % 360.0;
    }

    pub fn get_heading(&self) -> f64 {
        self.current_heading
    }

    fn adjust_heading(&mut self, degrees: f64) {
        self.current_heading = ((self.current_heading + degrees) % 360.0 + 360.0) % 360.0;
    }

    pub fn update(&mut self) -> Result<bool, Box<dyn Error>> {
        let position = self.read_toggle_switch();
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

    pub fn get_toggle_position(&self) -> SwitchPosition {
        self.last_toggle_position
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_gpio;

    #[test]
    fn test_heading_adjustment() {
        let mut heading = 0.0;
        heading = ((heading + 5.0) % 360.0 + 360.0) % 360.0;
        assert_eq!(heading, 5.0);

        heading = ((heading - 10.0) % 360.0 + 360.0) % 360.0;
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

    // Mocking GPIO
    #[test]
    fn test_toggle_switch_positions() -> Result<(), Box<dyn Error>> {
        mock_gpio::reset_mock_pins();

        let mut ui = UserInterface::new()?;

        // Test neutral
        mock_gpio::set_mock_pin_level(23, mock_gpio::Level::High);
        mock_gpio::set_mock_pin_level(24, mock_gpio::Level::High);
        ui.update()?;
        assert_eq!(ui.get_toggle_position(), SwitchPosition::Neutral);

        // Test LEFT
        mock_gpio::set_mock_pin_level(23, mock_gpio::Level::Low);
        thread::sleep(Duration::from_millis(50));
        ui.update()?;
        assert_eq!(ui.get_toggle_position(), SwitchPosition::Left);
        assert_eq!(ui.get_heading(), 355.0);

        // Back to neutral
        mock_gpio::set_mock_pin_level(23, mock_gpio::Level::High);
        thread::sleep(Duration::from_millis(50));
        ui.update()?;
        assert_eq!(ui.get_toggle_position(), SwitchPosition::Neutral);

        // Test RIGHT
        mock_gpio::set_mock_pin_level(24, mock_gpio::Level::Low);
        thread::sleep(Duration::from_millis(50));
        ui.update()?;
        assert_eq!(ui.get_toggle_position(), SwitchPosition::Right);
        assert_eq!(ui.get_heading(), 0.0);

        Ok(())
    }

    #[test]
    fn test_heading_wraparound() -> Result<(), Box<dyn Error>> {
        mock_gpio::reset_mock_pins();

        let mut ui = UserInterface::new()?;
        ui.set_heading(358.0);

        // Press RIGHT twice
        mock_gpio::set_mock_pin_level(24, mock_gpio::Level::Low);
        thread::sleep(Duration::from_millis(50));
        ui.update()?;

        mock_gpio::set_mock_pin_level(24, mock_gpio::Level::High);
        thread::sleep(Duration::from_millis(50));

        mock_gpio::set_mock_pin_level(24, mock_gpio::Level::Low);
        thread::sleep(Duration::from_millis(50));
        ui.update()?;

        assert_eq!(ui.get_heading(), 3.0);

        Ok(())
    }

    #[test]
    fn test_multiple_adjustments() -> Result<(), Box<dyn Error>> {
        mock_gpio::reset_mock_pins();

        let mut ui = UserInterface::new()?;
        ui.set_heading(90.0);

        for _ in 0..3 {
            mock_gpio::set_mock_pin_level(24, mock_gpio::Level::Low);
            thread::sleep(Duration::from_millis(50));
            ui.update()?;

            mock_gpio::set_mock_pin_level(24, mock_gpio::Level::High);
            thread::sleep(Duration::from_millis(50));
        }

        assert_eq!(ui.get_heading(), 105.0);

        Ok(())
    }
}
