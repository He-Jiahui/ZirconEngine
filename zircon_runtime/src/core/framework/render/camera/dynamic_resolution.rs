use crate::core::math::{Real, UVec2};
use serde::{Deserialize, Serialize};

pub const DEFAULT_DYNAMIC_RESOLUTION_SCALE: Real = 1.0;
pub const MIN_DYNAMIC_RESOLUTION_SCALE: Real = 0.1;
pub const MAX_DYNAMIC_RESOLUTION_SCALE: Real = 1.0;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderDynamicResolutionSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_dynamic_resolution_scale")]
    pub scale: Real,
}

impl Default for RenderDynamicResolutionSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            scale: DEFAULT_DYNAMIC_RESOLUTION_SCALE,
        }
    }
}

impl RenderDynamicResolutionSettings {
    pub const fn disabled() -> Self {
        Self {
            enabled: false,
            scale: DEFAULT_DYNAMIC_RESOLUTION_SCALE,
        }
    }

    pub fn fixed_scale(scale: Real) -> Self {
        Self {
            enabled: true,
            scale,
        }
    }

    pub fn clamped_scale(self) -> Real {
        if !self.enabled || !self.scale.is_finite() {
            return DEFAULT_DYNAMIC_RESOLUTION_SCALE;
        }
        self.scale
            .clamp(MIN_DYNAMIC_RESOLUTION_SCALE, MAX_DYNAMIC_RESOLUTION_SCALE)
    }

    pub fn apply_to_size(self, viewport_size: UVec2) -> UVec2 {
        let scale = self.clamped_scale();
        let width = ((viewport_size.x.max(1) as Real) * scale).round().max(1.0) as u32;
        let height = ((viewport_size.y.max(1) as Real) * scale).round().max(1.0) as u32;
        UVec2::new(width, height)
    }
}

pub(super) const fn default_dynamic_resolution_scale() -> Real {
    DEFAULT_DYNAMIC_RESOLUTION_SCALE
}
