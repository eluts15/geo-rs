// This file is only compiled during tests

use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Level {
    Low,
    High,
}

thread_local! {
    static MOCK_PINS: RefCell<HashMap<u8, Level>> = RefCell::new(HashMap::new());
}

pub struct InputPin {
    pin: u8,
}

impl InputPin {
    pub fn read(&self) -> Level {
        MOCK_PINS.with(|pins| *pins.borrow().get(&self.pin).unwrap_or(&Level::High))
    }
}

pub struct Gpio;

impl Gpio {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Gpio)
    }

    pub fn get(&self, pin: u8) -> Result<Pin, Box<dyn std::error::Error>> {
        Ok(Pin { pin })
    }
}

pub struct Pin {
    pin: u8,
}

impl Pin {
    pub fn into_input_pullup(self) -> InputPin {
        MOCK_PINS.with(|pins| {
            pins.borrow_mut().insert(self.pin, Level::High);
        });
        InputPin { pin: self.pin }
    }
}

// test helper function to set pin levels
pub fn set_mock_pin_level(pin: u8, level: Level) {
    MOCK_PINS.with(|pins| {
        pins.borrow_mut().insert(pin, level);
    });
}

// test helper to reset all pins
pub fn reset_mock_pins() {
    MOCK_PINS.with(|pins| {
        pins.borrow_mut().clear();
    });
}
