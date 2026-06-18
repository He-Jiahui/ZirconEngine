use super::*;

#[test]
fn property_row_label_keeps_label_and_value_visible() {
    let node = TemplatePaneNodeData {
        role: "Mount".into(),
        component_role: "property-row".into(),
        text: "Position".into(),
        value_text: "X 12.0   Y 3.5   Z -8.0".into(),
        ..TemplatePaneNodeData::default()
    };

    assert_eq!(
        template_node_label(&node, None),
        "Position    X 12.0   Y 3.5   Z -8.0"
    );
}
