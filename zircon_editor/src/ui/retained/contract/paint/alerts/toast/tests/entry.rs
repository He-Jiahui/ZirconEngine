use super::*;

#[test]
fn toast_prefers_the_projected_panel_radius() {
    let mut node = TemplatePaneNodeData::default();
    node.corner_radius = 14.0;

    assert_eq!(toast_surface_radius(&node, toast_metrics()), 14.0);
}
