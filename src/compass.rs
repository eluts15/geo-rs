/// Translates numeric headings to 4/8/16-point compass directions. (N, NE, E, etc.)
/// Represents a 16-point compass rose.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    N,   // north
    NNE, // north-northeast
    NE,  // northeast
    ENE, // east-northeast
    E,   // east
    ESE, // east-southeast
    SE,  // southeast
    SSE, // south-southeast
    S,   // south
    SSW, // south-southwest
    SW,  // southwest
    WSW, // west-southwest
    W,   // west
    WNW, // west-northwest
    NW,  // northwest
    NNW, // north-northwest
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

impl Direction {
    pub fn name(&self) -> &str {
        match self {
            Direction::N => "north",
            Direction::NNE => "north-northeast",
            Direction::NE => "northeast",
            Direction::ENE => "east-northeast",
            Direction::E => "east",
            Direction::ESE => "east-southeast",
            Direction::SE => "southeast",
            Direction::SSE => "south-Southeast",
            Direction::S => "south",
            Direction::SSW => "south-southwest",
            Direction::SW => "southwest",
            Direction::WSW => "west-southwest",
            Direction::W => "west",
            Direction::WNW => "west-northwest",
            Direction::NW => "northwest",
            Direction::NNW => "north-northwest",
        }
    }

    pub fn abbreviation(&self) -> &str {
        match self {
            Direction::N => "N",
            Direction::NNE => "NNE",
            Direction::NE => "NE",
            Direction::ENE => "ENE",
            Direction::E => "E",
            Direction::ESE => "ESE",
            Direction::SE => "SE",
            Direction::SSE => "SSE",
            Direction::S => "S",
            Direction::SSW => "SSW",
            Direction::SW => "SW",
            Direction::WSW => "WSW",
            Direction::W => "W",
            Direction::WNW => "WNW",
            Direction::NW => "NW",
            Direction::NNW => "NNW",
        }
    }
}

/// Converts the heading to a 16-point compass direction.
pub fn heading_to_direction_16point(heading: f64) -> (Direction, f64) {
    // normalize heading to 0-360 range
    let normalized = ((heading % 360.0) + 360.0) % 360.0;

    // each direction covers 22.5 degrees (360 / 16)
    let direction = match normalized {
        h if h < 11.25 => Direction::N,
        h if h < 33.75 => Direction::NNE,
        h if h < 56.25 => Direction::NE,
        h if h < 78.75 => Direction::ENE,
        h if h < 101.25 => Direction::E,
        h if h < 123.75 => Direction::ESE,
        h if h < 146.25 => Direction::SE,
        h if h < 168.75 => Direction::SSE,
        h if h < 191.25 => Direction::S,
        h if h < 213.75 => Direction::SSW,
        h if h < 236.25 => Direction::SW,
        h if h < 258.75 => Direction::WSW,
        h if h < 281.25 => Direction::W,
        h if h < 303.75 => Direction::WNW,
        h if h < 326.25 => Direction::NW,
        h if h < 348.75 => Direction::NNW,
        _ => Direction::N,
    };

    (direction, normalized)
}

/// Convert the heading to a 8-point compass direction.
pub fn heading_to_direction_8point(heading: f64) -> (Direction, f64) {
    // normalize heading to 0-360 range
    let normalized = ((heading % 360.0) + 360.0) % 360.0;

    // each direction covers 45 degrees (360 / 8)
    let direction = match normalized {
        h if h < 22.5 => Direction::N,
        h if h < 67.5 => Direction::NE,
        h if h < 112.5 => Direction::E,
        h if h < 157.5 => Direction::SE,
        h if h < 202.5 => Direction::S,
        h if h < 247.5 => Direction::SW,
        h if h < 292.5 => Direction::W,
        h if h < 337.5 => Direction::NW,
        _ => Direction::N,
    };
    (direction, normalized)
}

/// Convert the heading to a 4-point compass direction.
pub fn heading_to_direction_4point(heading: f64) -> (Direction, f64) {
    // normalize heading to 0-360 range
    let normalized = ((heading % 360.0) + 360.0) % 360.0;

    // each direction covers 90 degrees (360 / 4)
    let direction = match normalized {
        h if h < 45.0 => Direction::N,
        h if h < 135.0 => Direction::E,
        h if h < 225.0 => Direction::S,
        h if h < 315.0 => Direction::W,
        _ => Direction::N,
    };
    (direction, normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Direction::N), "N");
        assert_eq!(format!("{}", Direction::NNE), "NNE");
        assert_eq!(format!("{}", Direction::SE), "SE");
    }

    #[test]
    fn test_direction_names() {
        assert_eq!(Direction::N.name(), "north");
        assert_eq!(Direction::NE.name(), "northeast");
        assert_eq!(Direction::SSW.abbreviation(), "SSW");
    }

    #[test]
    fn test_heading_normalization() {
        let (dir, heading) = heading_to_direction_16point(370.0);
        assert_eq!(heading, 10.0);
        assert_eq!(dir, Direction::N);

        let (dir, heading) = heading_to_direction_16point(-10.0);
        assert_eq!(heading, 350.0);
        assert_eq!(dir, Direction::N);
    }

    #[test]
    fn test_heading_to_direction_16point() {
        assert_eq!(heading_to_direction_16point(0.0).0, Direction::N);
        assert_eq!(heading_to_direction_16point(22.5).0, Direction::NNE);
        assert_eq!(heading_to_direction_16point(45.0).0, Direction::NE);
        assert_eq!(heading_to_direction_16point(67.5).0, Direction::ENE);
        assert_eq!(heading_to_direction_16point(90.0).0, Direction::E);
        assert_eq!(heading_to_direction_16point(180.0).0, Direction::S);
        assert_eq!(heading_to_direction_16point(270.0).0, Direction::W);
        assert_eq!(heading_to_direction_16point(359.0).0, Direction::N);
    }

    #[test]
    fn test_heading_to_direction_8point() {
        assert_eq!(heading_to_direction_8point(0.0).0, Direction::N);
        assert_eq!(heading_to_direction_8point(45.0).0, Direction::NE);
        assert_eq!(heading_to_direction_8point(90.0).0, Direction::E);
        assert_eq!(heading_to_direction_8point(135.0).0, Direction::SE);
        assert_eq!(heading_to_direction_8point(180.0).0, Direction::S);
        assert_eq!(heading_to_direction_8point(225.0).0, Direction::SW);
        assert_eq!(heading_to_direction_8point(270.0).0, Direction::W);
        assert_eq!(heading_to_direction_8point(315.0).0, Direction::NW);
    }

    #[test]
    fn test_heading_to_direction_4point() {
        assert_eq!(heading_to_direction_4point(0.0).0, Direction::N);
        assert_eq!(heading_to_direction_4point(44.0).0, Direction::N);
        assert_eq!(heading_to_direction_4point(90.0).0, Direction::E);
        assert_eq!(heading_to_direction_4point(180.0).0, Direction::S);
        assert_eq!(heading_to_direction_4point(270.0).0, Direction::W);
    }
}
