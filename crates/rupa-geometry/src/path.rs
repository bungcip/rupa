//! Path geometry for Rupa.

use serde::{Deserialize, Serialize};

/// Path command types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PathCommand {
    /// Move to a new position without drawing
    MoveTo {
        /// X coordinate
        x: f32,
        /// Y coordinate
        y: f32,
    },
    /// Draw a line to a position
    LineTo {
        /// X coordinate
        x: f32,
        /// Y coordinate
        y: f32,
    },
    /// Draw a cubic Bézier curve
    CurveTo {
        /// First control point X
        x1: f32,
        /// First control point Y
        y1: f32,
        /// Second control point X
        x2: f32,
        /// Second control point Y
        y2: f32,
        /// End point X
        x: f32,
        /// End point Y
        y: f32,
    },
    /// Draw a quadratic Bézier curve
    QuadTo {
        /// Control point X
        x1: f32,
        /// Control point Y
        y1: f32,
        /// End point X
        x: f32,
        /// End point Y
        y: f32,
    },
    /// Close the current path
    ClosePath,
}

/// Path geometry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Path {
    /// List of path commands
    pub commands: Vec<PathCommand>,
}

impl Path {
    /// Create a new empty path
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Move to a position
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.commands.push(PathCommand::MoveTo { x, y });
    }

    /// Draw a line to a position
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.commands.push(PathCommand::LineTo { x, y });
    }

    /// Draw a cubic Bézier curve
    pub fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.commands.push(PathCommand::CurveTo {
            x1,
            y1,
            x2,
            y2,
            x,
            y,
        });
    }

    /// Draw a quadratic Bézier curve
    pub fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.commands.push(PathCommand::QuadTo { x1, y1, x, y });
    }

    /// Close the current path
    pub fn close(&mut self) {
        self.commands.push(PathCommand::ClosePath);
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}
