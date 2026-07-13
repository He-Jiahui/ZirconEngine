use super::super::super::data::FrameRect;
use super::metrics::{
    MAX_BOTTOM_GUTTER, MAX_LEFT_GUTTER, MAX_RIGHT_GUTTER, MAX_TOP_GUTTER, MIN_BOTTOM_GUTTER,
    MIN_LEFT_GUTTER, MIN_RIGHT_GUTTER, MIN_TOP_GUTTER, POINT_EDGE_INSET,
};

pub(super) struct SampleGridGeometry {
    pub outer: FrameRect,
    pub plot: FrameRect,
}

impl SampleGridGeometry {
    pub(super) fn from_frame(frame: &FrameRect) -> Self {
        let left = (frame.width * 0.12).clamp(MIN_LEFT_GUTTER, MAX_LEFT_GUTTER);
        let right = (frame.width * 0.035).clamp(MIN_RIGHT_GUTTER, MAX_RIGHT_GUTTER);
        let top = (frame.height * 0.09).clamp(MIN_TOP_GUTTER, MAX_TOP_GUTTER);
        let bottom = (frame.height * 0.15).clamp(MIN_BOTTOM_GUTTER, MAX_BOTTOM_GUTTER);
        let plot = FrameRect {
            x: frame.x + left,
            y: frame.y + top,
            width: (frame.width - left - right).max(1.0),
            height: (frame.height - top - bottom).max(1.0),
        };
        Self {
            outer: frame.clone(),
            plot,
        }
    }

    pub(super) fn x_for_value(&self, value: f32, min: f32, max: f32) -> f32 {
        self.plot.x + normalized(value, min, max) * self.plot.width
    }

    pub(super) fn y_for_value(&self, value: f32, min: f32, max: f32) -> f32 {
        self.plot.y + (1.0 - normalized(value, min, max)) * self.plot.height
    }

    pub(super) fn point_x_for_value(&self, value: f32, min: f32, max: f32) -> f32 {
        self.plot.x
            + POINT_EDGE_INSET
            + normalized(value, min, max) * (self.plot.width - POINT_EDGE_INSET * 2.0).max(0.0)
    }

    pub(super) fn point_y_for_value(&self, value: f32, min: f32, max: f32) -> f32 {
        self.plot.y
            + POINT_EDGE_INSET
            + (1.0 - normalized(value, min, max))
                * (self.plot.height - POINT_EDGE_INSET * 2.0).max(0.0)
    }
}

fn normalized(value: f32, min: f32, max: f32) -> f32 {
    if !value.is_finite() || !min.is_finite() || !max.is_finite() || max <= min {
        return 0.0;
    }
    ((value - min) / (max - min)).clamp(0.0, 1.0)
}
