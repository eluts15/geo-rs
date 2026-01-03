// This file is only compiled during tests

use std::cell::RefCell;
thread_local! {
    static MOCK_PWM_DUTY: RefCell<f64> = const { RefCell::new(0.5) }
}

pub struct Pwm {
    pin: u8,
}

impl Pwm {
    pub fn new(pin: u8) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Pwm { pin })
    }

    pub fn set_duty_cycle(&mut self, duty_cycle: f64) {
        MOCK_PWM_DUTY.with(|duty| {
            *duty.borrow_mut() = duty_cycle;
        });
        println!(
            "[Mock PWM {}] Duty cycle set to {:.4}",
            self.pin, duty_cycle
        );
    }

    pub fn disable(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("[Mock PWM {}] Disabled", self.pin);
        Ok(())
    }
}

// Test helper
pub fn get_mock_duty_cycle() -> f64 {
    MOCK_PWM_DUTY.with(|duty| *duty.borrow())
}
