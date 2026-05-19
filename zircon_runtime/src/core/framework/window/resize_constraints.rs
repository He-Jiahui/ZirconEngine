use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::constants::{DEFAULT_MIN_WINDOW_HEIGHT, DEFAULT_MIN_WINDOW_WIDTH};
use super::validation::{valid_max_window_axis, valid_window_axis};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowResizeConstraints {
    pub min_width: f32,
    pub min_height: f32,
    pub max_width: f32,
    pub max_height: f32,
}

#[derive(Deserialize, Serialize)]
struct WindowResizeConstraintsDto {
    min_width: f32,
    min_height: f32,
    max_width: Option<f32>,
    max_height: Option<f32>,
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

impl Serialize for WindowResizeConstraints {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        WindowResizeConstraintsDto {
            min_width: self.min_width,
            min_height: self.min_height,
            max_width: finite_axis(self.max_width),
            max_height: finite_axis(self.max_height),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for WindowResizeConstraints {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let dto = WindowResizeConstraintsDto::deserialize(deserializer)?;
        Ok(Self {
            min_width: dto.min_width,
            min_height: dto.min_height,
            max_width: dto.max_width.unwrap_or(f32::INFINITY),
            max_height: dto.max_height.unwrap_or(f32::INFINITY),
        }
        .validated())
    }
}

fn finite_axis(axis: f32) -> Option<f32> {
    if axis.is_finite() {
        Some(axis)
    } else {
        None
    }
}
