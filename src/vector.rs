use std::fmt;

use crate::position::Position;

#[derive(Clone, Copy, Debug)]
pub struct Vector {
    pub start: Position,
    pub heading: f64,
    pub distance: f64,
}

impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let end = self.end_position();
        write!(
            f,
            "Vector: {} -> {} (heading: {:.1}°, distance: {:.1}m)",
            self.start, end, self.heading, self.distance
        )
    }
}

impl Vector {
    pub fn new(start: Position, heading: f64, distance: f64) -> Self {
        Self {
            start,
            heading,
            distance,
        }
    }

    /// Get the end position of this vector.
    pub fn end_position(&self) -> Position {
        self.start.project(self.heading, self.distance)
    }

    /// Create a vector from the current position using the current heading.
    pub fn from_heading(position: Position, heading: f64, distance: f64) -> Self {
        Self::new(position, heading, distance)
    }
}
