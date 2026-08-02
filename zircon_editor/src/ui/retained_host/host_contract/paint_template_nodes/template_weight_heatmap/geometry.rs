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
        let legend_gutter =
            LEGEND_GAP + LEGEND_WIDTH + LEGEND_LABEL_GAP + legend_label_width.max(0.0);
        let plot_width = (frame.width - FRAME_INSET * 2.0 - legend_gutter).max(1.0);
        let plot = FrameRect {
            x: frame.x + FRAME_INSET,
            y: frame.y + FRAME_INSET,
            width: plot_width,
            height: (frame.height - FRAME_INSET * 2.0).max(1.0),
        };
        let legend = FrameRect {
            x: plot.x + plot.width + LEGEND_GAP,
            y: plot.y,
            width: LEGEND_WIDTH,
            height: plot.height,
        };
        Self {
            outer: frame.clone(),
            plot,
            legend,
        }
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
        let content_left = self.outer.x;
        let content_right = (self.outer.x + self.outer.width - FRAME_INSET).max(content_left + 1.0);
        let width = label_width
            .max(1.0)
            .min((content_right - content_left).max(1.0));
        let x = (self.legend.x + self.legend.width + LEGEND_LABEL_GAP)
            .clamp(content_left, content_right - width);
        FrameRect {
            x,
            y,
            width,
            height: line_height,
        }
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
}
