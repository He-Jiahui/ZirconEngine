#[test]
fn component_showcase_template_materializes_reference_drop_wells() {
    let template = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/template_pane.slint"
    ));

    assert!(
        template.contains("property <bool> reference_value_primitive"),
        "TemplatePane should classify AssetField, InstanceField, and ObjectField as reference wells"
    );
    assert!(
        template.contains("root.node.component_role == \"asset-field\""),
        "AssetField rows should render a retained drop well"
    );
    assert!(
        template.contains("root.node.component_role == \"instance-field\""),
        "InstanceField rows should render a retained drop well"
    );
    assert!(
        template.contains("root.node.component_role == \"object-field\""),
        "ObjectField rows should render a retained drop well"
    );
    assert!(
        template.contains("root.node.accepted_drag_payloads"),
        "Reference wells should surface accepted drag payload metadata"
    );
    assert!(
        template.contains("root.node.validation_message"),
        "Reference wells should surface rejected-drop and missing-reference validation messages"
    );
    assert!(
        template.contains("root.reference_value_primitive ? parent.width * 0.74"),
        "Reference action chips should move beside the drop well instead of covering the value"
    );
}
