use super::super::*;

#[test]
fn ui_v2_resolved_pseudo_state_uses_painter_selector_priority() {
    let mut document = v2_document("asset://ui/tests/resolved_runtime_style.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Button".to_string(),
            control_id: Some("ResolvedRuntimeButton".to_string()),
            classes: vec!["material".to_string()],
            state: BTreeMap::from([
                ("hovered".to_string(), Value::Boolean(true)),
                ("pressed".to_string(), Value::Boolean(true)),
            ]),
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_resolved_material".to_string(),
        rules: vec![
            style_rule("Button.material", [("background", "#101010")]),
            style_rule(
                "Button.material:resolved-pressed",
                [("background", "#303030")],
            ),
            style_rule(
                "Button.material:resolved-hovered",
                [("background", "#202020")],
            ),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let resolved = UiV2StyleResolver::resolve(&document, &compiled.arena).unwrap();

    assert_eq!(
        resolved.nodes["root"].self_values["background"].as_str(),
        Some("#303030")
    );
}

#[test]
fn ui_v2_resolved_pseudo_state_keeps_selection_identity_above_hover() {
    let mut selected_document = v2_document(
        "asset://ui/tests/resolved_selected_runtime_style.v2.ui",
        "root",
    );
    selected_document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "ListRow".to_string(),
            control_id: Some("ResolvedSelectedRow".to_string()),
            classes: vec!["material".to_string()],
            state: BTreeMap::from([
                ("hovered".to_string(), Value::Boolean(true)),
                ("drop_hovered".to_string(), Value::Boolean(true)),
                ("selected".to_string(), Value::Boolean(true)),
            ]),
            ..Default::default()
        },
    );
    selected_document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_resolved_selected_material".to_string(),
        rules: vec![
            style_rule("ListRow.material", [("background", "#101010")]),
            style_rule(
                "ListRow.material:resolved-hovered",
                [("background", "#202020")],
            ),
            style_rule(
                "ListRow.material:resolved-drop-hovered",
                [("background", "#252525")],
            ),
            style_rule(
                "ListRow.material:resolved-selected",
                [("background", "#303030")],
            ),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&selected_document).unwrap();
    let resolved = UiV2StyleResolver::resolve(&selected_document, &compiled.arena).unwrap();
    assert_eq!(
        resolved.nodes["root"].self_values["background"].as_str(),
        Some("#303030")
    );

    let mut checked_document = v2_document(
        "asset://ui/tests/resolved_checked_runtime_style.v2.ui",
        "root",
    );
    checked_document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Checkbox".to_string(),
            control_id: Some("ResolvedCheckedControl".to_string()),
            classes: vec!["material".to_string()],
            state: BTreeMap::from([
                ("hovered".to_string(), Value::Boolean(true)),
                ("drop_hovered".to_string(), Value::Boolean(true)),
                ("checked".to_string(), Value::Boolean(true)),
            ]),
            ..Default::default()
        },
    );
    checked_document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_resolved_checked_material".to_string(),
        rules: vec![
            style_rule("Checkbox.material", [("background", "#101010")]),
            style_rule(
                "Checkbox.material:resolved-hovered",
                [("background", "#202020")],
            ),
            style_rule(
                "Checkbox.material:resolved-drop-hovered",
                [("background", "#252525")],
            ),
            style_rule(
                "Checkbox.material:resolved-checked",
                [("background", "#404040")],
            ),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&checked_document).unwrap();
    let resolved = UiV2StyleResolver::resolve(&checked_document, &compiled.arena).unwrap();
    assert_eq!(
        resolved.nodes["root"].self_values["background"].as_str(),
        Some("#404040")
    );

    let mut open_document =
        v2_document("asset://ui/tests/resolved_open_runtime_style.v2.ui", "root");
    open_document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Dropdown".to_string(),
            control_id: Some("ResolvedOpenDropdown".to_string()),
            classes: vec!["material".to_string()],
            state: BTreeMap::from([
                ("hovered".to_string(), Value::Boolean(true)),
                ("drop_hovered".to_string(), Value::Boolean(true)),
                ("selected".to_string(), Value::Boolean(true)),
                ("popup_open".to_string(), Value::Boolean(true)),
            ]),
            ..Default::default()
        },
    );
    open_document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_resolved_open_material".to_string(),
        rules: vec![
            style_rule("Dropdown.material", [("background", "#101010")]),
            style_rule(
                "Dropdown.material:resolved-selected",
                [("background", "#303030")],
            ),
            style_rule(
                "Dropdown.material:resolved-open",
                [("background", "#505050")],
            ),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&open_document).unwrap();
    let resolved = UiV2StyleResolver::resolve(&open_document, &compiled.arena).unwrap();
    assert_eq!(
        resolved.nodes["root"].self_values["background"].as_str(),
        Some("#505050")
    );
}

#[test]
fn ui_v2_surface_runtime_resolved_pseudo_state_restyles_from_component_state_priority() {
    let mut document = v2_document(
        "asset://ui/tests/resolved_runtime_style_component.v2.ui",
        "root",
    );
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Button".to_string(),
            control_id: Some("ResolvedRuntimeButton".to_string()),
            classes: vec!["material".to_string()],
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_resolved_material".to_string(),
        rules: vec![
            style_rule("Button.material", [("background", "#101010")]),
            style_rule(
                "Button.material:resolved-pressed",
                [("background", "#303030")],
            ),
            style_rule(
                "Button.material:resolved-hovered",
                [("background", "#202020")],
            ),
            style_rule(
                "Button.material:resolved-disabled",
                [("background", "#707070")],
            ),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.resolved_runtime_style"),
        &document,
        &compiled,
    )
    .unwrap();
    let node_id = node_id_by_control_id(&surface, "ResolvedRuntimeButton");

    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#101010")
    );

    assert!(surface.component_states.set_hovered(node_id, true));
    surface.mark_component_state_render_dirty(node_id).unwrap();
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#202020")
    );

    assert!(surface.component_states.set_pressed(node_id, true));
    surface.mark_component_state_render_dirty(node_id).unwrap();
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#303030")
    );

    assert!(surface.component_states.set_disabled(node_id, true));
    surface.mark_component_state_render_dirty(node_id).unwrap();
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#707070")
    );
}

#[test]
fn ui_v2_surface_runtime_resolved_pseudo_state_keeps_selection_identity_above_hover() {
    let mut document = v2_document(
        "asset://ui/tests/resolved_selected_runtime_style_component.v2.ui",
        "root",
    );
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "ListRow".to_string(),
            control_id: Some("ResolvedRuntimeRow".to_string()),
            classes: vec!["material".to_string()],
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_resolved_row_material".to_string(),
        rules: vec![
            style_rule("ListRow.material", [("background", "#101010")]),
            style_rule(
                "ListRow.material:resolved-hovered",
                [("background", "#202020")],
            ),
            style_rule(
                "ListRow.material:resolved-drop-hovered",
                [("background", "#252525")],
            ),
            style_rule(
                "ListRow.material:resolved-selected",
                [("background", "#303030")],
            ),
            style_rule(
                "ListRow.material:resolved-checked",
                [("background", "#404040")],
            ),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.resolved_selected_runtime_style"),
        &document,
        &compiled,
    )
    .unwrap();
    let node_id = node_id_by_control_id(&surface, "ResolvedRuntimeRow");

    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#101010")
    );

    assert!(surface.component_states.set_hovered(node_id, true));
    surface.mark_component_state_render_dirty(node_id).unwrap();
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#202020")
    );

    assert!(surface.component_states.set_drop_hovered(node_id, true));
    surface.mark_component_state_render_dirty(node_id).unwrap();
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#252525")
    );

    assert!(surface.component_states.set_selected(node_id, true));
    surface.mark_component_state_render_dirty(node_id).unwrap();
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#303030")
    );

    assert!(surface.component_states.set_selected(node_id, false));
    assert!(surface.component_states.set_checked(node_id, true));
    surface.mark_component_state_render_dirty(node_id).unwrap();
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#404040")
    );
}
