pub mod compass;
pub mod gpio_input;
pub mod position;
pub mod vector;

use crate::position::Position;
use crate::vector::Vector;

#[cfg(test)]
pub mod mock_gpio;

pub struct GpsTracker {
    current_position: Option<Position>,
    current_heading: Option<f64>, // degrees
    current_speed: Option<f64>,   // knots
}

impl GpsTracker {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            current_position: None,
            current_heading: None,
            current_speed: None,
        }
    }

    pub fn get_current_position(&self) -> Option<Position> {
        self.current_position
    }

    pub fn get_current_heading(&self) -> Option<f64> {
        self.current_heading
    }

    pub fn get_current_speed(&self) -> Option<f64> {
        self.current_speed
    }

    pub fn update_position(&mut self, lat: f64, lon: f64) {
        self.current_position = Some(Position::new(lat, lon));
    }

    pub fn update_heading(&mut self, heading: f64) {
        self.current_heading = Some(heading);
    }

    pub fn update_speed(&mut self, speed: f64) {
        self.current_speed = Some(speed);
    }

    /// Create a vector from the current position inthe direction we're heading.
    pub fn get_forward_vector(&self, distance: f64) -> Option<Vector> {
        match (self.current_position, self.current_heading) {
            (Some(pos), Some(heading)) => Some(Vector::from_heading(pos, heading, distance)),
            _ => None,
        }
    }

    /// Get a vector from current position in a specific direction.
    pub fn get_vector_in_direction(&self, bearing: f64, distance: f64) -> Option<Vector> {
        self.current_position
            .map(|pos| Vector::new(pos, bearing, distance))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_calculation() {
        let pos1 = Position::new(48.0574, -123.1196); // Victoria, BC
        let pos2 = Position::new(48.4284, -123.3656); // Near Victoria

        let distance = pos1.distance_to(&pos2);
        println!("Distance: {:.2}m ({:.2}km)", distance, distance / 1000.0);

        // Verify it's a reasonable distance (between 40-50km)
        assert!(
            distance > 40000.0 && distance < 50000.0,
            "Distance should be ~42km, got {:.2}km",
            distance / 1000.0
        );
    }

    #[test]
    fn test_bearing_calculation() {
        let pos1 = Position::new(48.0, -123.0);
        let pos2 = Position::new(49.0, -123.0); // North

        let bearing = pos1.bearing_to(&pos2);
        assert!(bearing < 1.0); // Should be close to 0 (North)
    }

    #[test]
    fn test_projection() {
        let pos = Position::new(48.0, -123.0);
        let projected = pos.project(0.0, 1000.0); // 1km North

        println!("Original: {:.6}, {:.6}", pos.latitude, pos.longitude);
        println!(
            "Projected: {:.6}, {:.6}",
            projected.latitude, projected.longitude
        );
        println!("Lat diff: {:.6}", projected.latitude - pos.latitude);
        println!("Lon diff: {:.6}", projected.longitude - pos.longitude);

        // Going north should increase latitude
        assert!(
            projected.latitude > pos.latitude,
            "Latitude should increase when going north. Original: {}, Projected: {}",
            pos.latitude,
            projected.latitude
        );

        // Longitude should stay roughly the same (within 0.001 degrees)
        assert!(
            (projected.longitude - pos.longitude).abs() < 0.001,
            "Longitude should stay roughly the same. Diff: {}",
            (projected.longitude - pos.longitude).abs()
        );
    }
}
