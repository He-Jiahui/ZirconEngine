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
fn axis_label_text_uses_trimmed_declared_text_or_axis_fallback() {
    assert_eq!(
        axis_label_text(
            &text_node("WorkbenchTransformPositionAxisX", "  Position X  "),
            "X",
        ),
        "Position X"
    );
    assert_eq!(
        axis_label_text(&text_node("WorkbenchTransformRotationAxisY", "   "), "Y"),
        "Y"
    );
}
