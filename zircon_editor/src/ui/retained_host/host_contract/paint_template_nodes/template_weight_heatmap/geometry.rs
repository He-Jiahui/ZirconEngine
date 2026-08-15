use super::super::super::data::FrameRect;

const FRAME_INSET: f32 = 3.0;
const LEGEND_GAP: f32 = 5.0;
const LEGEND_WIDTH: f32 = 8.0;
const LEGEND_LABEL_GAP: f32 = 2.0;

pub(super) struct WeightHeatmapGeometry {
    pub outer: FrameRect,
    pub plot: FrameRect,
    pub legend: FrameRect,
}

impl WeightHeatmapGeometry {
    pub(super) fn from_frame(frame: &FrameRect, legend_label_width: f32) -> Self {
        let outer = FrameRect {
            x: finite_coordinate(frame.x),
            y: finite_coordinate(frame.y),
            width: finite_non_negative(frame.width),
            height: finite_non_negative(frame.height),
        };
        let legend_gutter =
            LEGEND_GAP + LEGEND_WIDTH + LEGEND_LABEL_GAP + finite_non_negative(legend_label_width);
        let plot_width = finite_non_negative(outer.width - FRAME_INSET * 2.0 - legend_gutter);
        let plot_height = finite_non_negative(outer.height - FRAME_INSET * 2.0);
        let plot = FrameRect {
            x: outer.x + FRAME_INSET,
            y: outer.y + FRAME_INSET,
            width: plot_width,
            height: plot_height,
        };
        let legend = FrameRect {
            x: plot.x + plot.width + LEGEND_GAP,
            y: plot.y,
            width: if plot_width > f32::EPSILON && plot_height > f32::EPSILON {
                LEGEND_WIDTH
            } else {
                0.0
            },
            height: plot_height,
        };
        Self {
            outer,
            plot,
            legend,
        }
    }

    pub(super) fn is_drawable(&self) -> bool {
        self.outer.width > f32::EPSILON
            && self.outer.height > f32::EPSILON
            && self.plot.width > f32::EPSILON
            && self.plot.height > f32::EPSILON
    }

    pub(super) fn x_for_normalized(&self, value: f32) -> f32 {
        self.plot.x + value.clamp(0.0, 1.0) * self.plot.width
    }

    pub(super) fn y_for_normalized(&self, value: f32) -> f32 {
        self.plot.y + (1.0 - value.clamp(0.0, 1.0)) * self.plot.height
    }

    pub(super) fn legend_label_frame(
        &self,
        label_width: f32,
        y: f32,
        line_height: f32,
    ) -> FrameRect {
        if !self.is_drawable() {
            return FrameRect {
                x: self.outer.x,
                y: self.outer.y,
                width: 0.0,
                height: 0.0,
            };
        }
        let content_left = self.outer.x;
        let content_right = self.outer.x + self.outer.width - FRAME_INSET;
        let content_top = self.outer.y + FRAME_INSET;
        let content_bottom = self.outer.y + self.outer.height - FRAME_INSET;
        let width =
            finite_non_negative(label_width).min(finite_non_negative(content_right - content_left));
        let y = finite_coordinate(y).clamp(content_top, content_bottom);
        let x = (self.legend.x + self.legend.width + LEGEND_LABEL_GAP)
            .clamp(content_left, content_right - width);
        FrameRect {
            x,
            y,
            width,
            height: finite_non_negative(line_height).min(finite_non_negative(content_bottom - y)),
        }
    }
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn finite_coordinate(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::WeightHeatmapGeometry;
    use crate::ui::retained_host::host_contract::data::FrameRect;

    #[test]
    fn heatmap_geometry_reserves_measured_legend_label_width_inside_content_bounds() {
        let frame = FrameRect {
            x: 10.0,
            y: 4.0,
            width: 160.0,
            height: 90.0,
        };
        let label_width = 42.0;
        let geometry = WeightHeatmapGeometry::from_frame(&frame, label_width);
        let label = geometry.legend_label_frame(label_width, 8.0, 10.0);

        assert!(geometry.plot.width < 130.0);
        assert_eq!(label.x + label.width, frame.x + frame.width - 3.0);

        let narrow_frame = FrameRect {
            x: 10.0,
            y: 4.0,
            width: 20.0,
            height: 90.0,
        };
        let narrow_geometry = WeightHeatmapGeometry::from_frame(&narrow_frame, label_width);
        let narrow_label = narrow_geometry.legend_label_frame(label_width, 8.0, 10.0);
        assert!(narrow_label.x >= narrow_frame.x);
        assert!(narrow_label.x + narrow_label.width <= narrow_frame.x + narrow_frame.width - 3.0);
    }

    #[test]
    fn collapsed_or_invalid_frames_do_not_materialize_heatmap_geometry() {
        let geometry = WeightHeatmapGeometry::from_frame(
            &FrameRect {
                x: f32::NAN,
                y: f32::INFINITY,
                width: 0.0,
                height: f32::NEG_INFINITY,
            },
            f32::NAN,
        );

        assert!(!geometry.is_drawable());
        assert_eq!(geometry.plot.width, 0.0);
        assert_eq!(geometry.plot.height, 0.0);
        assert_eq!(geometry.legend.width, 0.0);
        assert_eq!(geometry.legend_label_frame(20.0, 4.0, 10.0).width, 0.0);
    }
}
