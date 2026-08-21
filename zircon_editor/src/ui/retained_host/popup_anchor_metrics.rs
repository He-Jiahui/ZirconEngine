#[cfg(test)]
use crate::ui::retained_host::METRICS;
use crate::ui::retained_host::{current_host_metrics, HostControlMetrics};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PopupAnchorMetrics {
    pub(crate) edge_margin: f32,
    pub(crate) anchor_gap: f32,
    pub(crate) render_gap: f32,
}

pub(crate) const POPUP_EDGE_MARGIN: f32 = 8.0;
pub(crate) const POPUP_ANCHOR_GAP: f32 = 3.0;
pub(crate) const TOOLBAR_POPUP_RENDER_GAP: f32 = 4.0;

pub(crate) const SLATE_POPUP_ANCHOR_METRICS: PopupAnchorMetrics = PopupAnchorMetrics {
    edge_margin: POPUP_EDGE_MARGIN,
    anchor_gap: POPUP_ANCHOR_GAP,
    render_gap: TOOLBAR_POPUP_RENDER_GAP,
};

const COMMAND_PALETTE_GUTTER_FRACTION: f32 = 0.04;
const COMMAND_PALETTE_TOP_FRACTION: f32 = 0.1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PopupAnchorFrame {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
}

pub(crate) fn toolbar_popup_render_gap() -> f32 {
    current_popup_anchor_metrics().render_gap
}

pub(crate) fn current_popup_anchor_metrics() -> PopupAnchorMetrics {
    popup_anchor_metrics_from_host(current_host_metrics())
}

fn popup_anchor_metrics_from_host(metrics: HostControlMetrics) -> PopupAnchorMetrics {
    let edge_margin = metrics.gap_m.max(0.0);
    let render_gap = metrics.gap_s.max(0.0);
    PopupAnchorMetrics {
        edge_margin,
        anchor_gap: (render_gap - metrics.border_width.max(0.0)).max(0.0),
        render_gap,
    }
}

/// Computes a palette-wide anchor from the mounted workbench bounds.
pub(crate) fn command_palette_anchor_frame(width: f32, height: f32) -> PopupAnchorFrame {
    command_palette_anchor_frame_with_metrics(width, height, current_host_metrics())
}

fn command_palette_anchor_frame_with_metrics(
    width: f32,
    height: f32,
    metrics: HostControlMetrics,
) -> PopupAnchorFrame {
    let width = width.max(1.0);
    let height = height.max(1.0);
    let gap = metrics.gap_l.max(0.0);
    let default_height = metrics.control_default_height.max(gap);
    let large_height = metrics.control_large_height.max(0.0);
    let gutter = (width * COMMAND_PALETTE_GUTTER_FRACTION).clamp(gap, default_height);
    let top = (height * COMMAND_PALETTE_TOP_FRACTION).clamp(large_height, large_height + gap * 2.0);
    PopupAnchorFrame {
        x: gutter,
        y: top,
        width: (width - gutter * 2.0).max(1.0),
    }
}

pub(crate) fn clamp_popup_x_to_bounds(
    authored_x: f32,
    bounds_x: f32,
    bounds_width: f32,
    popup_width: f32,
) -> f32 {
    clamp_popup_x_to_bounds_with_metrics(
        authored_x,
        bounds_x,
        bounds_width,
        popup_width,
        current_popup_anchor_metrics(),
    )
}

fn clamp_popup_x_to_bounds_with_metrics(
    authored_x: f32,
    bounds_x: f32,
    bounds_width: f32,
    popup_width: f32,
    metrics: PopupAnchorMetrics,
) -> f32 {
    if bounds_width <= 0.0 || popup_width <= 0.0 {
        return authored_x.max(bounds_x);
    }

    let margin = metrics.edge_margin.min(bounds_width * 0.5);
    let min_x = bounds_x + margin;
    let max_x = bounds_x + bounds_width - margin - popup_width;
    if max_x >= min_x {
        authored_x.clamp(min_x, max_x)
    } else {
        bounds_x.max(bounds_x + bounds_width - popup_width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_popup_x_to_bounds_preserves_shared_edge_margin_when_space_allows() {
        assert_eq!(
            clamp_popup_x_to_bounds_with_metrics(
                120.0,
                0.0,
                160.0,
                80.0,
                SLATE_POPUP_ANCHOR_METRICS
            ),
            72.0
        );
        assert_eq!(
            clamp_popup_x_to_bounds_with_metrics(2.0, 0.0, 160.0, 80.0, SLATE_POPUP_ANCHOR_METRICS),
            8.0
        );
        assert_eq!(
            clamp_popup_x_to_bounds_with_metrics(
                24.0,
                20.0,
                160.0,
                80.0,
                SLATE_POPUP_ANCHOR_METRICS
            ),
            28.0
        );
    }

    #[test]
    fn popup_anchor_metrics_follow_projected_density_and_control_tokens() {
        let mut metrics = METRICS;
        metrics.gap_s = 7.0;
        metrics.gap_m = 14.0;
        metrics.border_width = 2.0;

        assert_eq!(
            popup_anchor_metrics_from_host(metrics),
            PopupAnchorMetrics {
                edge_margin: 14.0,
                anchor_gap: 5.0,
                render_gap: 7.0,
            }
        );
    }

    #[test]
    fn popup_x_clamp_uses_the_projected_edge_margin() {
        let metrics = PopupAnchorMetrics {
            edge_margin: 16.0,
            anchor_gap: 5.0,
            render_gap: 7.0,
        };

        assert_eq!(
            clamp_popup_x_to_bounds_with_metrics(120.0, 0.0, 160.0, 80.0, metrics),
            64.0
        );
        assert_eq!(
            clamp_popup_x_to_bounds_with_metrics(2.0, 0.0, 160.0, 80.0, metrics),
            16.0
        );
    }

    #[test]
    fn command_palette_anchor_uses_responsive_workbench_bounds() {
        assert_eq!(
            command_palette_anchor_frame_with_metrics(360.0, 640.0, METRICS),
            PopupAnchorFrame {
                x: 14.4,
                y: 64.0,
                width: 331.2,
            }
        );
        assert_eq!(
            command_palette_anchor_frame_with_metrics(1200.0, 900.0, METRICS),
            PopupAnchorFrame {
                x: 32.0,
                y: 72.0,
                width: 1136.0,
            }
        );
    }

    #[test]
    fn command_palette_anchor_uses_projected_control_metrics_for_its_bounds() {
        let mut metrics = METRICS;
        metrics.gap_l = 20.0;
        metrics.control_default_height = 36.0;
        metrics.control_large_height = 48.0;

        assert_eq!(
            command_palette_anchor_frame_with_metrics(360.0, 640.0, metrics),
            PopupAnchorFrame {
                x: 20.0,
                y: 64.0,
                width: 320.0,
            }
        );
        assert_eq!(
            command_palette_anchor_frame_with_metrics(1200.0, 900.0, metrics),
            PopupAnchorFrame {
                x: 36.0,
                y: 88.0,
                width: 1128.0,
            }
        );
    }

    #[test]
    fn command_palette_anchor_normalizes_inverted_custom_control_bounds() {
        let mut metrics = METRICS;
        metrics.gap_l = 20.0;
        metrics.control_default_height = 12.0;
        metrics.control_large_height = 10.0;

        assert_eq!(
            command_palette_anchor_frame_with_metrics(360.0, 640.0, metrics),
            PopupAnchorFrame {
                x: 20.0,
                y: 50.0,
                width: 320.0,
            }
        );
    }
}
