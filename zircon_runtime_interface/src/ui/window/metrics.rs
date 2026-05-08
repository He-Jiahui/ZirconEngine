use serde::{Deserialize, Serialize};

use crate::ui::layout::UiSize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiWindowPixelSize {
    pub width: u32,
    pub height: u32,
}

impl UiWindowPixelSize {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiWindowPixelPosition {
    pub x: i32,
    pub y: i32,
}

impl UiWindowPixelPosition {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Logical and physical window metrics share one DTO so DPI changes can mark
/// layout metrics dirty without implying input-state mutation.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiWindowMetrics {
    #[serde(default)]
    pub logical_size: UiSize,
    #[serde(default)]
    pub physical_size: UiWindowPixelSize,
    #[serde(default = "default_scale_factor")]
    pub scale_factor: f64,
}

impl UiWindowMetrics {
    pub const fn new(
        logical_size: UiSize,
        physical_size: UiWindowPixelSize,
        scale_factor: f64,
    ) -> Self {
        Self {
            logical_size,
            physical_size,
            scale_factor,
        }
    }
}

impl Default for UiWindowMetrics {
    fn default() -> Self {
        Self {
            logical_size: UiSize::default(),
            physical_size: UiWindowPixelSize::default(),
            scale_factor: default_scale_factor(),
        }
    }
}

const fn default_scale_factor() -> f64 {
    1.0
}
