use super::*;

#[test]
fn popup_background_uses_the_panel_radius_tier() {
    let metrics = current_host_metrics();
    let node = TemplatePaneNodeData::default();

    assert_eq!(popup_background_style(&node).radius, metrics.radius_panel);
}

#[test]
fn popup_background_prefers_the_projected_panel_radius() {
    let mut node = TemplatePaneNodeData::default();
    node.corner_radius = 14.0;

    assert_eq!(popup_background_style(&node).radius, 14.0);
}
