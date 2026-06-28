use serde::{Deserialize, Serialize};

pub const PIXEL_SCROLL_LINE_DELTA_SCALE: f32 = 0.1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseScrollUnit {
    #[default]
    Line,
    Pixel,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MouseWheelEvent {
    pub unit: MouseScrollUnit,
    pub x: f32,
    pub y: f32,
}

impl MouseWheelEvent {
    pub const fn new(unit: MouseScrollUnit, x: f32, y: f32) -> Self {
        Self { unit, x, y }
    }

    pub const fn lines(x: f32, y: f32) -> Self {
        Self::new(MouseScrollUnit::Line, x, y)
    }

    pub const fn pixels(x: f32, y: f32) -> Self {
        Self::new(MouseScrollUnit::Pixel, x, y)
    }

    pub fn vertical_line_delta(self) -> f32 {
        match self.unit {
            MouseScrollUnit::Line => self.y,
            MouseScrollUnit::Pixel => self.y * PIXEL_SCROLL_LINE_DELTA_SCALE,
        }
    }
}
