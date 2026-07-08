use super::super::*;

#[test]
fn ui_v2_inline_style_overrides_cascade_values_in_style_overrides() {
    let mut document = v2_document("asset://ui/tests/style_override_priority.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Button".to_string(),
            control_id: Some("OverrideButton".to_string()),
            classes: vec!["material-button".to_string()],
            style: UiV2StyleDeclarationBlock {
                self_values: BTreeMap::from([(
                    "button_variant".to_string(),
                    Value::String("outlined".to_string()),
                )]),
                slot: BTreeMap::new(),
            },
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "style_override_priority".to_string(),
        rules: vec![style_rule(
            "Button.material-button",
            [("button_variant", "contained")],
        )],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.style_override_priority"),
        &document,
        &compiled,
    )
    .unwrap();
    let node_id = node_id_by_control_id(&surface, "OverrideButton");
    let metadata = surface
        .tree
        .nodes
        .get(&node_id)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();

    assert_eq!(
        metadata.attributes["button_variant"].as_str(),
        Some("contained")
    );
    assert_eq!(
        metadata.style_overrides["button_variant"].as_str(),
        Some("outlined")
    );
}
