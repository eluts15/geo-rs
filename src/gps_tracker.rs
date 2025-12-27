use crate::compass_sensor::CompassSensor;
use crate::position::Position;
use crate::vector::Vector;

pub struct GpsTracker {
    current_position: Option<Position>,
    current_heading: Option<f64>, // degrees
    current_speed: Option<f64>,   // knots
    num_satellites: Option<u8>,
}

impl GpsTracker {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            current_position: None,
            current_heading: None,
            current_speed: None,
            num_satellites: None,
        }
    }

    pub fn get_current_position(&self) -> Option<Position> {
        self.current_position
    }

    pub fn update_position(&mut self, lat: f64, lon: f64) {
        self.current_position = Some(Position::new(lat, lon));
    }

    pub fn get_current_heading(&self) -> Option<f64> {
        self.current_heading
    }

    pub fn update_heading(&mut self, heading: f64) {
        self.current_heading = Some(heading);
    }

    pub fn get_num_satellites(&self) -> Option<u8> {
        self.num_satellites
    }

    pub fn update_satellites(&mut self, num_sats: u8) {
        self.num_satellites = Some(num_sats);
    }

    pub fn get_current_speed(&self) -> Option<f64> {
        self.current_speed
    }

    pub fn update_speed(&mut self, speed: f64) {
        self.current_speed = Some(speed);
    }

    /// This is where we're currently heading.
    pub fn get_forward_vector(&self, distance: f64) -> Option<Vector> {
        match (self.current_position, self.current_heading) {
            (Some(pos), Some(heading)) => Some(Vector::from_heading(pos, heading, distance)),
            _ => None,
        }
    }

    /// Get a vector from current position in a specific heading (this is where we want to go).
    pub fn get_vector_to_azimuth(&self, heading: f64, distance: f64) -> Option<Vector> {
        self.current_position
            .map(|pos| Vector::new(pos, heading, distance))
    }

    pub fn get_current_heading_with_compass(&self, compass: &mut CompassSensor) -> Option<f64> {
        // Prefer GPS heading when moving
        if let Some(gps_heading) = self.current_heading {
            return Some(gps_heading);
        }

        // Fall back to compass when stationary
        compass.read_heading().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gps_tracker_initialization() {
        let tracker = GpsTracker::new();
        assert!(tracker.get_current_position().is_none());
        assert!(tracker.get_current_heading().is_none());
        assert!(tracker.get_current_speed().is_none());
        assert!(tracker.get_num_satellites().is_none());
    }

    #[test]
    fn test_gps_tracker_updates() {
        let mut tracker = GpsTracker::new();

        tracker.update_position(48.0, -123.0);
        assert!(tracker.get_current_position().is_some());

        tracker.update_heading(90.0);
        assert_eq!(tracker.get_current_heading(), Some(90.0));

        tracker.update_speed(5.5);
        assert_eq!(tracker.get_current_speed(), Some(5.5));

        tracker.update_satellites(8);
        assert_eq!(tracker.get_num_satellites(), Some(8));
    }

    #[test]
    fn test_vector_generation() {
        let mut tracker = GpsTracker::new();
        tracker.update_position(48.0, -123.0);
        tracker.update_heading(90.0);

        let forward = tracker.get_forward_vector(100.0);
        assert!(forward.is_some());

        let directional = tracker.get_vector_to_azimuth(45.0, 100.0);
        assert!(directional.is_some());
    }
}
