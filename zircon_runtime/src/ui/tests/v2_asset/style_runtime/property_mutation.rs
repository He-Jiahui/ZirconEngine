use super::super::*;

#[test]
fn ui_v2_surface_property_mutation_restyles_checked_and_disabled_pseudo_state() {
    let mut document = v2_document("asset://ui/tests/runtime_style_property.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Toggle".to_string(),
            control_id: Some("RuntimeToggle".to_string()),
            classes: vec!["material-toggle".to_string()],
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_property_material".to_string(),
        rules: vec![
            style_rule("Toggle.material-toggle", [("background", "#101010")]),
            style_rule(
                "Toggle.material-toggle:checked",
                [("background", "#225533")],
            ),
            style_rule(
                "Toggle.material-toggle:disabled",
                [("foreground", "#778899")],
            ),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_style_property"),
        &document,
        &compiled,
    )
    .unwrap();
    let node_id = node_id_by_control_id(&surface, "RuntimeToggle");

    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#101010")
    );

    let checked = surface
        .mutate_property(UiPropertyMutationRequest::new(
            node_id,
            "checked",
            UiValue::Bool(true),
        ))
        .unwrap();
    assert_eq!(checked.status, UiPropertyMutationStatus::Accepted);
    assert!(checked.invalidation.dirty.render);
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#225533")
    );

    surface.clear_dirty_flags();
    let disabled = surface
        .mutate_property(UiPropertyMutationRequest::new(
            node_id,
            "enabled",
            UiValue::Bool(false),
        ))
        .unwrap();
    assert_eq!(disabled.status, UiPropertyMutationStatus::Accepted);
    assert!(disabled.invalidation.dirty.input);
    assert!(disabled.invalidation.dirty.render);
    assert_eq!(
        runtime_color_attr(&surface, node_id, "foreground"),
        Some("#778899")
    );
    let dirty = surface.tree.nodes.get(&node_id).unwrap().dirty;
    assert!(dirty.input);
    assert!(dirty.render);
    assert!(!dirty.layout);
}

#[test]
fn ui_v2_surface_property_mutation_restyles_focused_pseudo_state() {
    let mut document = v2_document(
        "asset://ui/tests/runtime_style_focus_property.v2.ui",
        "root",
    );
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Dropdown".to_string(),
            control_id: Some("RuntimeDropdown".to_string()),
            classes: vec!["runtime-dropdown".to_string()],
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_focus_property_material".to_string(),
        rules: vec![
            style_rule("Dropdown.runtime-dropdown", [("background", "#101010")]),
            style_rule(
                "Dropdown.runtime-dropdown:focus",
                [("background", "#151b1f")],
            ),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_style_focus_property"),
        &document,
        &compiled,
    )
    .unwrap();
    let node_id = node_id_by_control_id(&surface, "RuntimeDropdown");

    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#101010")
    );

    let focused = surface
        .mutate_property(UiPropertyMutationRequest::new(
            node_id,
            "focused",
            UiValue::Bool(true),
        ))
        .unwrap();

    assert_eq!(focused.status, UiPropertyMutationStatus::Accepted);
    assert!(surface
        .component_state(node_id)
        .is_some_and(|state| state.flags.focused));
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#151b1f")
    );
}

#[test]
fn ui_v2_surface_property_mutation_updates_runtime_style_baseline_metadata() {
    let mut document = v2_document("asset://ui/tests/runtime_style_visibility.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Panel".to_string(),
            control_id: Some("RuntimePanel".to_string()),
            classes: vec!["runtime-panel".to_string()],
            props: BTreeMap::from([(
                "visibility".to_string(),
                Value::String("collapsed".to_string()),
            )]),
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_visibility_material".to_string(),
        rules: vec![style_rule(
            "Panel.runtime-panel:hover",
            [("background", "#202020")],
        )],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_style_visibility"),
        &document,
        &compiled,
    )
    .unwrap();
    let node_id = node_id_by_control_id(&surface, "RuntimePanel");
    surface.tree.nodes.get_mut(&node_id).unwrap().visibility = UiVisibility::Collapsed;

    let visibility = surface
        .mutate_property(UiPropertyMutationRequest::new(
            node_id,
            "visibility",
            UiValue::String("visible".to_string()),
        ))
        .unwrap();

    assert_eq!(visibility.status, UiPropertyMutationStatus::Accepted);
    assert_eq!(
        surface.tree.nodes.get(&node_id).unwrap().visibility,
        UiVisibility::Visible
    );
    assert_eq!(
        runtime_attr(&surface, node_id, "visibility"),
        Some("visible")
    );
}
