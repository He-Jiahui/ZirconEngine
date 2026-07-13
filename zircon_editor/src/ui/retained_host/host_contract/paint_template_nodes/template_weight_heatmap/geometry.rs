use super::super::super::data::FrameRect;

const FRAME_INSET: f32 = 3.0;
const LEGEND_GUTTER: f32 = 25.0;

pub(super) struct WeightHeatmapGeometry {
    pub outer: FrameRect,
    pub plot: FrameRect,
    pub legend: FrameRect,
}

impl WeightHeatmapGeometry {
    pub(super) fn from_frame(frame: &FrameRect) -> Self {
        let plot_width = (frame.width - FRAME_INSET * 2.0 - LEGEND_GUTTER).max(1.0);
        let plot = FrameRect {
            x: frame.x + FRAME_INSET,
            y: frame.y + FRAME_INSET,
            width: plot_width,
            height: (frame.height - FRAME_INSET * 2.0).max(1.0),
        };
        let legend = FrameRect {
            x: plot.x + plot.width + 5.0,
            y: plot.y,
            width: 8.0,
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
}
