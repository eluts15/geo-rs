use std::fmt;

/// Earth's radius in meters.
const EARTH_RADIUS: f64 = 6371000.0;

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

    /// Calculate heading from this position to another position.
    /// Returns the heading in degrees (0-360, where 0 is North).
    pub fn heading_to(&self, other: &Position) -> f64 {
        let lat_from = self.latitude.to_radians();
        let lat_to = other.latitude.to_radians();
        let delta_lon = (other.longitude - self.longitude).to_radians();

        let y = delta_lon.sin() * lat_to.cos();
        let x = lat_from.cos() * lat_to.sin() - lat_from.sin() * lat_to.cos() * delta_lon.cos();

        let heading = y.atan2(x).to_degrees();

        (heading + 360.0) % 360.0
    }

    /// Calculate distance to another position using Haversine formula.
    /// Read more here: https://en.wikipedia.org/wiki/Haversine_formula
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

    /// Project a position forward by a given distance and heading
    /// heading: degrees (0-360, where 0 is North)
    /// distance: meters
    pub fn project(&self, heading: f64, distance: f64) -> Position {
        let lat1 = self.latitude.to_radians();
        let lon1 = self.longitude.to_radians();
        let brng = heading.to_radians();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_new_and_display() {
        let pos = Position::new(48.057440, -123.119625);

        assert_eq!(pos.latitude, 48.057440);
        assert_eq!(pos.longitude, -123.119625);

        let display = format!("{}", pos);
        assert_eq!(display, "(48.057440°, -123.119625°)");

        println!("Position created: {}", pos);
    }

    #[test]
    fn test_lat_lon_to_radians() {
        use std::f64::consts::PI;

        // Test known conversions
        let pos1 = Position::new(0.0, 0.0); // Equator, Prime Meridian
        assert_eq!(pos1.latitude.to_radians(), 0.0);
        assert_eq!(pos1.longitude.to_radians(), 0.0);

        let pos2 = Position::new(90.0, 180.0); // North Pole, International Date Line
        assert!((pos2.latitude.to_radians() - PI / 2.0).abs() < 1e-10);
        assert!((pos2.longitude.to_radians() - PI).abs() < 1e-10);

        let pos3 = Position::new(-90.0, -180.0); // South Pole
        assert!((pos3.latitude.to_radians() + PI / 2.0).abs() < 1e-10);
        assert!((pos3.longitude.to_radians() + PI).abs() < 1e-10);

        let pos4 = Position::new(45.0, 90.0); // 45°N, 90°E
        assert!((pos4.latitude.to_radians() - PI / 4.0).abs() < 1e-10);
        assert!((pos4.longitude.to_radians() - PI / 2.0).abs() < 1e-10);

        // Test our actual GPS coordinates
        let pos5 = Position::new(48.057440, -123.119625);
        let lat_rad = pos5.latitude.to_radians();
        let lon_rad = pos5.longitude.to_radians();

        println!("Position: {}", pos5);
        println!("Latitude in radians: {:.6}", lat_rad);
        println!("Longitude in radians: {:.6}", lon_rad);

        // Verify conversion back
        assert!((lat_rad.to_degrees() - 48.057440).abs() < 1e-10);
        assert!((lon_rad.to_degrees() - (-123.119625)).abs() < 1e-10);
    }
}
