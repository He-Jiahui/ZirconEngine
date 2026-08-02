use super::super::super::data::FrameRect;
use super::metrics::TimelineStripMetrics;
use crate::ui::retained_host::host_contract::paint_geometry::bounded_extent;

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
        if !has_paintable_timeline_strip_extent(frame) {
            return empty_timeline_strip_geometry(frame);
        }
        let inner = inset_frame(frame, metrics.inset);
        if inner.width <= 0.0 || inner.height <= 0.0 {
            return empty_timeline_strip_geometry(frame);
        }
        let gap = bounded_extent(metrics.inset).min(inner.height);
        let footer_height = bounded_extent(metrics.footer_height).min(inner.height * 0.28);
        let plot_height = (inner.height - footer_height - gap).max(0.0);
        let ruler_height = bounded_extent(metrics.ruler_height).min(plot_height * 0.35);
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
            .max(0.0)
            .min((plot.height - ruler.height).max(0.0) * 0.62);
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

pub(super) fn has_paintable_timeline_strip_extent(frame: &FrameRect) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}

fn inset_frame(frame: &FrameRect, inset: f32) -> FrameRect {
    let width = bounded_extent(frame.width);
    let height = bounded_extent(frame.height);
    let inset = bounded_extent(inset);
    let inset_x = inset.min(width * 0.5);
    let inset_y = inset.min(height * 0.5);
    FrameRect {
        x: frame.x + inset_x,
        y: frame.y + inset_y,
        width: (width - inset_x * 2.0).max(0.0),
        height: (height - inset_y * 2.0).max(0.0),
    }
}

fn empty_timeline_strip_geometry(frame: &FrameRect) -> TimelineStripGeometry {
    let empty = FrameRect {
        x: frame.x,
        y: frame.y,
        width: 0.0,
        height: 0.0,
    };
    TimelineStripGeometry {
        outer: frame.clone(),
        ruler: empty.clone(),
        plot: empty.clone(),
        track: empty.clone(),
        footer: empty,
    }
}

fn normalized_time(time: f32, duration: f32) -> f32 {
    if !time.is_finite() || !duration.is_finite() || duration <= 0.0 {
        return 0.0;
    }
    (time / duration).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapsed_timeline_strip_has_no_drawable_regions() {
        let geometry = TimelineStripGeometry::from_frame(
            &FrameRect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 120.0,
            },
            super::super::metrics::timeline_metrics(),
        );

        for region in [
            geometry.ruler,
            geometry.plot,
            geometry.track,
            geometry.footer,
        ] {
            assert_eq!(region.width, 0.0);
            assert_eq!(region.height, 0.0);
        }
    }
}
