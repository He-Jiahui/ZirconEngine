use super::*;

#[test]
fn clamp_popup_x_to_bounds_preserves_shared_edge_margin_when_space_allows() {
    assert_eq!(
        clamp_popup_x_to_bounds_with_metrics(120.0, 0.0, 160.0, 80.0, SLATE_POPUP_ANCHOR_METRICS),
        72.0
    );
    assert_eq!(
        clamp_popup_x_to_bounds_with_metrics(2.0, 0.0, 160.0, 80.0, SLATE_POPUP_ANCHOR_METRICS),
        8.0
    );
    assert_eq!(
        clamp_popup_x_to_bounds_with_metrics(24.0, 20.0, 160.0, 80.0, SLATE_POPUP_ANCHOR_METRICS),
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
