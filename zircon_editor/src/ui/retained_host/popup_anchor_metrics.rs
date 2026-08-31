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
}
