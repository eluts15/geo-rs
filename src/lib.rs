use std::fmt;

const EARTH_RADIUS: f64 = 6371000.0; // in meters

#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub latitude: f64,
    pub longitude: f64,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.6}°, {:.6}°)", self.latitude, self.longitude)
    }
}

impl Position {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
        }
    }

    /// Calculate bearing from this position to another position.
    /// Returns the bearing in degrees (0-360, where 0 is North).
    pub fn bearing_to(&self, other: &Position) -> f64 {
        let lat_from = self.latitude.to_radians();
        let lat_to = other.latitude.to_radians();
        let delta_lon = (other.longitude - self.longitude).to_radians();

        let y = delta_lon.sin() * lat_to.cos();
        let x = lat_from.cos() * lat_to.sin() - lat_from.sin() * lat_to.cos() * delta_lon.cos();

        let bearing = y.atan2(x).to_degrees();

        (bearing + 360.0) % 360.0
    }

    /// Calculate distance to another position using Haversine formula.
    /// Returns the distance in meters.
    pub fn distance_to(&self, other: &Position) -> f64 {
        let lat_from = self.latitude.to_radians();
        let lat_to = other.latitude.to_radians();
        let delta_lat = (other.latitude - self.latitude).to_radians();
        let delta_lon = (other.longitude - self.longitude).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat_from.cos() * lat_to.cos() * (delta_lon / 2.0).sin().powi(2);

        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS * c
    }

    /// Project a position forward by a given distance and bearing
    /// bearing: degrees (0-360, where 0 is North)
    /// distance: meters
    pub fn project(&self, bearing: f64, distance: f64) -> Position {
        const EARTH_RADIUS: f64 = 6371000.0; // meters

        let lat1 = self.latitude.to_radians();
        let lon1 = self.longitude.to_radians();
        let brng = bearing.to_radians();
        let angular_distance = distance / EARTH_RADIUS;

        let lat2 = (lat1.sin() * angular_distance.cos()
            + lat1.cos() * angular_distance.sin() * brng.cos())
        .asin();

        let lon2 = lon1
            + (brng.sin() * angular_distance.sin() * lat1.cos())
                .atan2(angular_distance.cos() - lat1.sin() * lat2.sin());

        // Normalize longitude to -180 to 180
        let lon2_normalized = ((lon2.to_degrees() + 180.0) % 360.0) - 180.0;

        Position::new(lat2.to_degrees(), lon2_normalized)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Vector {
    pub start: Position,
    pub bearing: f64,
    pub distance: f64,
}

impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let end = self.end_position();
        write!(
            f,
            "Vector: {} -> {} (bearing: {:.1}°, distance: {:.1}m)",
            self.start, end, self.bearing, self.distance
        )
    }
}

impl Vector {
    pub fn new(start: Position, bearing: f64, distance: f64) -> Self {
        Self {
            start,
            bearing,
            distance,
        }
    }

    /// Get the end position of this vector.
    pub fn end_position(&self) -> Position {
        self.start.project(self.bearing, self.distance)
    }

    /// Create a vector from the current position using the current heading.
    pub fn from_heading(position: Position, heading: f64, distance: f64) -> Self {
        Self::new(position, heading, distance)
    }
}

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
