use crate::core::math::{Real, UVec2};

pub const DEFAULT_CAMERA_EXPOSURE_EV100: Real = 9.7;
pub const DEFAULT_CAMERA_MSAA_SAMPLES: u32 = 1;

pub const fn default_viewport_aspect_ratio() -> Real {
    16.0 / 9.0
}

pub fn aspect_ratio_from_viewport_size(viewport_size: UVec2) -> Real {
    viewport_size.x.max(1) as Real / viewport_size.y.max(1) as Real
}

pub(super) const fn default_true() -> bool {
    true
}

pub(super) const fn default_camera_exposure_ev100() -> Real {
    DEFAULT_CAMERA_EXPOSURE_EV100
}

pub(super) const fn default_camera_msaa_samples() -> u32 {
    DEFAULT_CAMERA_MSAA_SAMPLES
}
