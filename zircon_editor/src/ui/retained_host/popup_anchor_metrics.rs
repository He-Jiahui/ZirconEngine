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
    SLATE_POPUP_ANCHOR_METRICS.render_gap
}

pub(crate) fn clamp_popup_x_to_bounds(
    authored_x: f32,
    bounds_x: f32,
    bounds_width: f32,
    popup_width: f32,
) -> f32 {
    if bounds_width <= 0.0 || popup_width <= 0.0 {
        return authored_x.max(bounds_x);
    }

    let margin = SLATE_POPUP_ANCHOR_METRICS
        .edge_margin
        .min(bounds_width * 0.5);
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
        assert_eq!(clamp_popup_x_to_bounds(120.0, 0.0, 160.0, 80.0), 72.0);
        assert_eq!(clamp_popup_x_to_bounds(2.0, 0.0, 160.0, 80.0), 8.0);
        assert_eq!(clamp_popup_x_to_bounds(24.0, 20.0, 160.0, 80.0), 28.0);
    }
}
