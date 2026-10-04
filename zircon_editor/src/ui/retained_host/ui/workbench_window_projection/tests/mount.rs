use super::*;

#[test]
fn physical_projection_scales_geometry_and_visual_metrics_but_not_semantic_values() {
    let mut node = host_contract::TemplatePaneNodeData::default();
    node.frame = host_contract::TemplateNodeFrameData {
        x: 8.0,
        y: 12.0,
        width: 80.0,
        height: 40.0,
    };
    node.font_size = 12.0;
    node.corner_radius = 3.0;
    node.border_width = 1.0;
    node.value_number = 0.75;
    node.value_percent = 0.75;
    node.layout_second_cell_offset_x = 28.0;
    node.layout_third_cell_offset_x = 5.0;
    node.layout_padding_left = 8.0;
    node.layout_padding_right = 8.0;
    node.layout_padding_top = 4.0;
    node.layout_padding_bottom = 4.0;
    node.layout_spacing = 4.0;
    node.transition_duration_ms = 120;
    node.button_style.width = StyleDimension::Fixed(24.0);
    node.button_style.element.corner_radius = 2.0;

    let projected =
        project_node_into_physical_mount(node, Some(UiFrame::new(100.0, 50.0, 400.0, 200.0)), 2.0);

    assert_eq!(projected.frame.x, 116.0);
    assert_eq!(projected.frame.y, 74.0);
    assert_eq!(projected.frame.width, 160.0);
    assert_eq!(projected.font_size, 24.0);
    assert_eq!(projected.corner_radius, 6.0);
    assert_eq!(projected.border_width, 2.0);
    assert_eq!(projected.value_number, 0.75);
    assert_eq!(projected.value_percent, 0.75);
    assert_eq!(projected.layout_second_cell_offset_x, 28.0);
    assert_eq!(projected.layout_third_cell_offset_x, 5.0);
    assert_eq!(projected.layout_padding_left, 16.0);
    assert_eq!(projected.layout_padding_right, 16.0);
    assert_eq!(projected.layout_padding_top, 8.0);
    assert_eq!(projected.layout_padding_bottom, 8.0);
    assert_eq!(projected.layout_spacing, 8.0);
    assert_eq!(projected.transition_duration_ms, 120);
    assert_eq!(projected.button_style.width, StyleDimension::Fixed(48.0));
    assert_eq!(projected.button_style.element.corner_radius, 4.0);
}
