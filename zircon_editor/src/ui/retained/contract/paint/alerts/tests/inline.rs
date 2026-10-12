use super::*;

#[test]
fn inline_alert_prefers_the_projected_panel_radius() {
    let mut node = TemplatePaneNodeData::default();
    node.corner_radius = 14.0;

    assert_eq!(alert_surface_radius(&node, alert_metrics()), 14.0);
}
