use serde::{Deserialize, Serialize};

use super::constants::{DEFAULT_MIN_WINDOW_HEIGHT, DEFAULT_MIN_WINDOW_WIDTH};
use super::validation::{valid_max_window_axis, valid_window_axis};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowResizeConstraints {
    pub min_width: f32,
    pub min_height: f32,
    pub max_width: f32,
    pub max_height: f32,
}

impl WindowResizeConstraints {
    pub fn validated(self) -> Self {
        let min_width = valid_window_axis(self.min_width);
        let min_height = valid_window_axis(self.min_height);
        let max_width = valid_max_window_axis(self.max_width).max(min_width);
        let max_height = valid_max_window_axis(self.max_height).max(min_height);

        Self {
            min_width,
            min_height,
            max_width,
            max_height,
        }
    }
}

impl Default for WindowResizeConstraints {
    fn default() -> Self {
        Self {
            min_width: DEFAULT_MIN_WINDOW_WIDTH,
            min_height: DEFAULT_MIN_WINDOW_HEIGHT,
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
        }
    }
}
