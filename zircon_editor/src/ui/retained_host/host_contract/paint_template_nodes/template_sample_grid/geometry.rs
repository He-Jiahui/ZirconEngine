use super::super::super::data::FrameRect;
use super::metrics::{
    MAX_BOTTOM_GUTTER, MAX_LEFT_GUTTER, MAX_RIGHT_GUTTER, MAX_TOP_GUTTER, MIN_BOTTOM_GUTTER,
    MIN_LEFT_GUTTER, MIN_RIGHT_GUTTER, MIN_TOP_GUTTER, POINT_EDGE_INSET,
};
use crate::ui::retained_host::host_contract::paint_geometry::bounded_extent;

pub(super) struct SampleGridGeometry {
    pub outer: FrameRect,
    pub plot: FrameRect,
}

impl SampleGridGeometry {
    pub(super) fn from_frame(frame: &FrameRect) -> Self {
        let width = bounded_extent(frame.width);
        let height = bounded_extent(frame.height);
        if width <= 0.0 || height <= 0.0 {
            return Self {
                outer: frame.clone(),
                plot: FrameRect {
                    x: frame.x,
                    y: frame.y,
                    width: 0.0,
                    height: 0.0,
                },
            };
        }
        let left = (width * 0.12)
            .clamp(MIN_LEFT_GUTTER, MAX_LEFT_GUTTER)
            .min(width);
        let right = (width * 0.035)
            .clamp(MIN_RIGHT_GUTTER, MAX_RIGHT_GUTTER)
            .min((width - left).max(0.0));
        let top = (height * 0.14)
            .clamp(MIN_TOP_GUTTER, MAX_TOP_GUTTER)
            .min(height);
        let bottom = (height * 0.04)
            .clamp(MIN_BOTTOM_GUTTER, MAX_BOTTOM_GUTTER)
            .min((height - top).max(0.0));
        let plot = FrameRect {
            x: frame.x + left,
            y: frame.y + top,
            width: (width - left - right).max(0.0),
            height: (height - top - bottom).max(0.0),
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
        let inset = POINT_EDGE_INSET.min(self.plot.width * 0.5);
        self.plot.x + inset + normalized(value, min, max) * (self.plot.width - inset * 2.0).max(0.0)
    }

    pub(super) fn point_y_for_value(&self, value: f32, min: f32, max: f32) -> f32 {
        let inset = POINT_EDGE_INSET.min(self.plot.height * 0.5);
        self.plot.y
            + inset
            + (1.0 - normalized(value, min, max)) * (self.plot.height - inset * 2.0).max(0.0)
    }
}

pub(super) fn has_paintable_sample_grid_extent(frame: &FrameRect) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}

fn normalized(value: f32, min: f32, max: f32) -> f32 {
    if !value.is_finite() || !min.is_finite() || !max.is_finite() || max <= min {
        return 0.0;
    }
    ((value - min) / (max - min)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapsed_sample_grid_has_no_plot_area() {
        let geometry = SampleGridGeometry::from_frame(&FrameRect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 160.0,
        });

        assert_eq!(geometry.plot.width, 0.0);
        assert_eq!(geometry.plot.height, 0.0);
    }

    #[test]
    fn compact_plot_centers_points_instead_of_applying_an_oversized_inset() {
        let geometry = SampleGridGeometry {
            outer: FrameRect::default(),
            plot: FrameRect {
                x: 10.0,
                y: 20.0,
                width: 8.0,
                height: 10.0,
            },
        };

        for value in [0.0, 0.5, 1.0] {
            assert_eq!(geometry.point_x_for_value(value, 0.0, 1.0), 14.0);
            assert_eq!(geometry.point_y_for_value(value, 0.0, 1.0), 25.0);
        }
    }
}
