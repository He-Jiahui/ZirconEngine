use super::*;

fn text_node(control_id: &str, text: &str) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: "Label".into(),
        text: text.into(),
        ..TemplatePaneNodeData::default()
    }
}

#[test]
fn axis_label_text_style_keeps_runtime_plain_text_route() {
    let metrics = AxisLabelMetrics {
        font_size: 12.0,
        line_height: 15.0,
        link_lobe_width: 6.0,
        link_lobe_height: 7.0,
        link_lobe_radius: 3.0,
        link_overlap: 2.0,
        link_connector_width: 1.0,
    };
    let style =
        axis_label_text_command_style(&text_node("WorkbenchTransformRotationAxisZ", "Z"), &metrics);

    assert_eq!(style.font_size, 12.0);
    assert_eq!(style.line_height, 15.0);
    assert_eq!(style.paint_style, UiTextRunPaintStyle::default());
}
