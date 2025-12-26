/// Represents a 16-point compass rose.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Azimuth {
    N,   // North
    NNE, // North-NorthEast
    NE,  // North-East
    ENE, // East-NorthEast
    E,   // East
    ESE, // East-SouthEast
    SE,  // South-East
    SSE, // South-SouthEast
    S,   // South
    SSW, // South-SouthWest
    SW,  // South-West
    WSW, // West-SouthWest
    W,   // West
    WNW, // West-NorthWest
    NW,  // North-West
    NNW, // North-NorthWest
}

impl Azimuth {
    pub fn name(&self) -> &str {
        match self {
            Azimuth::N => "North",
            Azimuth::NNE => "North-NorthEast",
            Azimuth::NE => "North-East",
            Azimuth::ENE => "East-NorthEast",
            Azimuth::E => "East",
            Azimuth::ESE => "East-SouthEast",
            Azimuth::SE => "South-East",
            Azimuth::SSE => "South-SouthEast",
            Azimuth::S => "South",
            Azimuth::SSW => "South-SouthWest",
            Azimuth::SW => "South-West",
            Azimuth::WSW => "West-SouthWest",
            Azimuth::W => "West",
            Azimuth::WNW => "West-NorthWest",
            Azimuth::NW => "North-West",
            Azimuth::NNW => "North-NorthWest",
        }
    }

    pub fn abbreviation(&self) -> &str {
        match self {
            Azimuth::N => "N",
            Azimuth::NNE => "NNE",
            Azimuth::NE => "NE",
            Azimuth::ENE => "ENE",
            Azimuth::E => "E",
            Azimuth::ESE => "ESE",
            Azimuth::SE => "SE",
            Azimuth::SSE => "SSE",
            Azimuth::S => "S",
            Azimuth::SSW => "SSW",
            Azimuth::SW => "SW",
            Azimuth::WSW => "WSW",
            Azimuth::W => "W",
            Azimuth::WNW => "WNW",
            Azimuth::NW => "NW",
            Azimuth::NNW => "NNW",
        }
    }
}

impl std::fmt::Display for Azimuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

/// convert the heading to a 16-point compass.
pub fn heading_to_azimuth_16point(heading: f64) -> (Azimuth, f64) {
    // normalize heading to 0-360 range
    let normalized = ((heading % 360.0) + 360.0) % 360.0;

    // each azimuth covers 22.5 degrees (360 / 16)
    let azimuth = match normalized {
        h if h < 11.25 => Azimuth::N,
        h if h < 33.75 => Azimuth::NNE,
        h if h < 56.25 => Azimuth::NE,
        h if h < 78.75 => Azimuth::ENE,
        h if h < 101.25 => Azimuth::E,
        h if h < 123.75 => Azimuth::ESE,
        h if h < 146.25 => Azimuth::SE,
        h if h < 168.75 => Azimuth::SSE,
        h if h < 191.25 => Azimuth::S,
        h if h < 213.75 => Azimuth::SSW,
        h if h < 236.25 => Azimuth::SW,
        h if h < 258.75 => Azimuth::WSW,
        h if h < 281.25 => Azimuth::W,
        h if h < 303.75 => Azimuth::WNW,
        h if h < 326.25 => Azimuth::NW,
        h if h < 348.75 => Azimuth::NNW,
        _ => Azimuth::N,
    };

    (azimuth, normalized)
}

/// Convert the heading to a 8-point compass.
pub fn heading_to_azimuth_8point(heading: f64) -> (Azimuth, f64) {
    // normalize heading to 0-360 range
    let normalized = ((heading % 360.0) + 360.0) % 360.0;

    // each azimuth covers 45 degrees (360 / 8)
    let azimuth = match normalized {
        h if h < 22.5 => Azimuth::N,
        h if h < 67.5 => Azimuth::NE,
        h if h < 112.5 => Azimuth::E,
        h if h < 157.5 => Azimuth::SE,
        h if h < 202.5 => Azimuth::S,
        h if h < 247.5 => Azimuth::SW,
        h if h < 292.5 => Azimuth::W,
        h if h < 337.5 => Azimuth::NW,
        _ => Azimuth::N,
    };
    (azimuth, normalized)
}

/// Convert the heading to a 4-point compass.
pub fn heading_to_azimuth_4point(heading: f64) -> (Azimuth, f64) {
    // normalize heading to 0-360 range
    let normalized = ((heading % 360.0) + 360.0) % 360.0;

    // each azimuth covers 90 degrees (360 / 4)
    let azimuth = match normalized {
        h if h < 45.0 => Azimuth::N,
        h if h < 135.0 => Azimuth::E,
        h if h < 225.0 => Azimuth::S,
        h if h < 315.0 => Azimuth::W,
        _ => Azimuth::N,
    };
    (azimuth, normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading_to_azimuth_16point() {
        assert_eq!(heading_to_azimuth_16point(0.0).0, Azimuth::N);
        assert_eq!(heading_to_azimuth_16point(22.5).0, Azimuth::NNE);
        assert_eq!(heading_to_azimuth_16point(45.0).0, Azimuth::NE);
        assert_eq!(heading_to_azimuth_16point(67.5).0, Azimuth::ENE);
        assert_eq!(heading_to_azimuth_16point(90.0).0, Azimuth::E);
        assert_eq!(heading_to_azimuth_16point(180.0).0, Azimuth::S);
        assert_eq!(heading_to_azimuth_16point(270.0).0, Azimuth::W);
        assert_eq!(heading_to_azimuth_16point(359.0).0, Azimuth::N);
    }

    #[test]
    fn test_heading_to_azimuth_8point() {
        assert_eq!(heading_to_azimuth_8point(0.0).0, Azimuth::N);
        assert_eq!(heading_to_azimuth_8point(45.0).0, Azimuth::NE);
        assert_eq!(heading_to_azimuth_8point(90.0).0, Azimuth::E);
        assert_eq!(heading_to_azimuth_8point(135.0).0, Azimuth::SE);
        assert_eq!(heading_to_azimuth_8point(180.0).0, Azimuth::S);
        assert_eq!(heading_to_azimuth_8point(225.0).0, Azimuth::SW);
        assert_eq!(heading_to_azimuth_8point(270.0).0, Azimuth::W);
        assert_eq!(heading_to_azimuth_8point(315.0).0, Azimuth::NW);
    }

    #[test]
    fn test_heading_to_azimuth_4point() {
        assert_eq!(heading_to_azimuth_4point(0.0).0, Azimuth::N);
        assert_eq!(heading_to_azimuth_4point(44.0).0, Azimuth::N);
        assert_eq!(heading_to_azimuth_4point(90.0).0, Azimuth::E);
        assert_eq!(heading_to_azimuth_4point(180.0).0, Azimuth::S);
        assert_eq!(heading_to_azimuth_4point(270.0).0, Azimuth::W);
    }

    #[test]
    fn test_azimuth_names() {
        assert_eq!(Azimuth::N.name(), "North");
        assert_eq!(Azimuth::NE.name(), "North-East");
        assert_eq!(Azimuth::SSW.abbreviation(), "SSW");
    }

    #[test]
    fn test_heading_normalization() {
        let (dir, heading) = heading_to_azimuth_16point(370.0);
        assert_eq!(heading, 10.0);
        assert_eq!(dir, Azimuth::N);

        let (dir, heading) = heading_to_azimuth_16point(-10.0);
        assert_eq!(heading, 350.0);
        assert_eq!(dir, Azimuth::N);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Azimuth::N), "N");
        assert_eq!(format!("{}", Azimuth::NNE), "NNE");
        assert_eq!(format!("{}", Azimuth::SE), "SE");
    }
}
