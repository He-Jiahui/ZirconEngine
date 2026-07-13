use super::super::super::data::FrameRect;
use super::metrics::TimelineStripMetrics;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct TimelineStripGeometry {
    pub outer: FrameRect,
    pub ruler: FrameRect,
    pub plot: FrameRect,
    pub track: FrameRect,
    pub footer: FrameRect,
}

impl TimelineStripGeometry {
    pub(super) fn from_frame(frame: &FrameRect, metrics: TimelineStripMetrics) -> Self {
        let inner = inset_frame(frame, metrics.inset);
        let footer_height = metrics.footer_height.min(inner.height * 0.28).max(1.0);
        let plot_height = (inner.height - footer_height - metrics.inset).max(1.0);
        let ruler_height = metrics.ruler_height.min(plot_height * 0.35).max(1.0);
        let plot = FrameRect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: plot_height,
        };
        let ruler = FrameRect {
            x: plot.x,
            y: plot.y,
            width: plot.width,
            height: ruler_height,
        };
        let track_height = metrics
            .track_height
            .min((plot.height - ruler.height) * 0.62)
            .max(1.0);
        let track = FrameRect {
            x: plot.x,
            y: ruler.y + ruler.height,
            width: plot.width,
            height: track_height,
        };
        let footer = FrameRect {
            x: inner.x,
            y: inner.y + inner.height - footer_height,
            width: inner.width,
            height: footer_height,
        };
        Self {
            outer: frame.clone(),
            ruler,
            plot,
            track,
            footer,
        }
    }

    pub(super) fn x_for_time(&self, time: f32, duration: f32) -> f32 {
        self.plot.x + normalized_time(time, duration) * self.plot.width
    }
}

fn inset_frame(frame: &FrameRect, inset: f32) -> FrameRect {
    FrameRect {
        x: frame.x + inset,
        y: frame.y + inset,
        width: (frame.width - inset * 2.0).max(1.0),
        height: (frame.height - inset * 2.0).max(1.0),
    }
}

fn normalized_time(time: f32, duration: f32) -> f32 {
    if !time.is_finite() || !duration.is_finite() || duration <= 0.0 {
        return 0.0;
    }
    (time / duration).clamp(0.0, 1.0)
}
