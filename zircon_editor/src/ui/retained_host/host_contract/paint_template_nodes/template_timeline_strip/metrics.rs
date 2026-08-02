use super::super::super::paint_theme::{HostControlMetrics, current_host_metrics};

const OUTER_RADIUS_SCALE: f32 = 0.5;
const KEY_RADIUS_SCALE: f32 = 3.0;
const PLAYHEAD_WIDTH_SCALE: f32 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct TimelineStripMetrics {
    pub outer_radius: f32,
    pub border_width: f32,
    pub inset: f32,
    pub ruler_height: f32,
    pub track_height: f32,
    pub footer_height: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub key_radius: i32,
    pub playhead_width: f32,
}

pub(super) fn timeline_metrics() -> TimelineStripMetrics {
    timeline_metrics_from_host(current_host_metrics())
}

pub(super) fn timeline_metrics_from_host(host: HostControlMetrics) -> TimelineStripMetrics {
    TimelineStripMetrics {
        outer_radius: host.radius_control * OUTER_RADIUS_SCALE,
        border_width: host.border_width,
        inset: host.gap_s,
        ruler_height: host.row_height,
        track_height: host.row_height,
        footer_height: host.line_height(host.font_small) + host.gap_s * 2.0,
        font_size: host.font_small,
        line_height: host.line_height(host.font_small),
        key_radius: (host.border_width * KEY_RADIUS_SCALE).round().max(2.0) as i32,
        playhead_width: (host.border_width * PLAYHEAD_WIDTH_SCALE).max(1.0),
    }
}
