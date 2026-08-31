use crate::core::math::{Real, UVec2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderViewportRect {
    pub physical_position: UVec2,
    pub physical_size: UVec2,
    pub depth_min: Real,
    pub depth_max: Real,
}

impl Default for RenderViewportRect {
    fn default() -> Self {
        Self {
            physical_position: UVec2::ZERO,
            physical_size: UVec2::new(1, 1),
            depth_min: 0.0,
            depth_max: 1.0,
        }
    }
}

impl RenderViewportRect {
    pub fn new(physical_position: UVec2, physical_size: UVec2) -> Self {
        Self {
            physical_position,
            physical_size,
            ..Self::default()
        }
    }

    pub fn clamped_to_size(mut self, target_size: UVec2) -> Self {
        self.physical_position.x =
            clamp_viewport_axis_position(self.physical_position.x, target_size.x);
        self.physical_position.y =
            clamp_viewport_axis_position(self.physical_position.y, target_size.y);
        self.physical_size.x = self
            .physical_size
            .x
            .min(target_size.x.saturating_sub(self.physical_position.x));
        self.physical_size.y = self
            .physical_size
            .y
            .min(target_size.y.saturating_sub(self.physical_position.y));
        self
    }
}

fn clamp_viewport_axis_position(position: u32, target: u32) -> u32 {
    if target == 0 {
        0
    } else {
        position.min(target - 1)
    }
}
