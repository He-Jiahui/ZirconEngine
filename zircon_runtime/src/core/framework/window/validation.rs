use super::constants::{DEFAULT_WINDOW_SCALE_FACTOR, MIN_WINDOW_AXIS};

pub(super) fn valid_window_axis(axis: f32) -> f32 {
    if axis.is_finite() {
        axis.max(MIN_WINDOW_AXIS)
    } else {
        MIN_WINDOW_AXIS
    }
}

pub(super) fn valid_max_window_axis(axis: f32) -> f32 {
    if axis.is_nan() {
        MIN_WINDOW_AXIS
    } else {
        axis.max(MIN_WINDOW_AXIS)
    }
}

pub(super) fn valid_scale_factor(scale_factor: f32) -> f32 {
    if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        DEFAULT_WINDOW_SCALE_FACTOR
    }
}
