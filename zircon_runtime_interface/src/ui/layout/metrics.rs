use serde::{Deserialize, Serialize};

use super::{UiPixelSnapping, UiSize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UiFlowDirection {
    #[default]
    LeftToRight,
    RightToLeft,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiLayoutMetrics {
    #[serde(default)]
    pub logical_size: UiSize,
    #[serde(default)]
    pub physical_size: UiSize,
    #[serde(default = "default_scale")]
    pub dpi_scale: f32,
    #[serde(default = "default_scale")]
    pub font_scale: f32,
    #[serde(default = "default_scale")]
    pub layout_scale: f32,
    #[serde(default)]
    pub flow_direction: UiFlowDirection,
    #[serde(default)]
    pub pixel_snapping: UiPixelSnapping,
}

impl Default for UiLayoutMetrics {
    fn default() -> Self {
        Self {
            logical_size: UiSize::default(),
            physical_size: UiSize::default(),
            dpi_scale: 1.0,
            font_scale: 1.0,
            layout_scale: 1.0,
            flow_direction: UiFlowDirection::LeftToRight,
            pixel_snapping: UiPixelSnapping::Enabled,
        }
    }
}

const fn default_scale() -> f32 {
    1.0
}
