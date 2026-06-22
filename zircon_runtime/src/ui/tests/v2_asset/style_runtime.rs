use super::*;

#[test]
fn ui_v2_style_specificity_and_pseudo_state_are_resolved() {
    let mut document = v2_document("asset://ui/tests/style.v2.ui", "root");
    document.tokens.insert(
        "material.primary".to_string(),
        Value::String("#6750a4".to_string()),
    );
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Button".to_string(),
            control_id: Some("RunButton".to_string()),
            classes: vec!["primary".to_string()],
            state: BTreeMap::from([("hovered".to_string(), Value::Boolean(true))]),
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "editor_material".to_string(),
        rules: vec![
            style_rule("Button", [("fg", "#111111")]),
            style_rule(".primary", [("fg", "#222222")]),
            style_rule("Button.primary:hover", [("fg", "$material.primary")]),
            style_rule("#RunButton", [("radius", "6")]),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let resolved = UiV2StyleResolver::resolve(&document, &compiled.arena).unwrap();
    let root = resolved.nodes.get("root").unwrap();

    assert_eq!(root.self_values["fg"].as_str(), Some("#6750a4"));
    assert_eq!(root.self_values["radius"].as_str(), Some("6"));
}

#[test]
fn ui_v2_style_resolver_can_resolve_theme_tokens_when_registry_is_supplied() {
    let mut document = v2_document("asset://ui/tests/theme_style.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Button".to_string(),
            control_id: Some("ThemeButton".to_string()),
            classes: vec!["primary".to_string()],
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "theme_material".to_string(),
        rules: vec![style_rule(
            "Button.primary",
            [
                ("background", "$theme.palette.accent"),
                ("foreground", "var(theme.palette.text.primary)"),
            ],
        )],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let unresolved = UiV2StyleResolver::resolve(&document, &compiled.arena).unwrap();
    assert_eq!(
        unresolved.nodes["root"].self_values["background"].as_str(),
        Some("$theme.palette.accent")
    );

    let resolved = UiV2StyleResolver::resolve_with_theme(
        &document,
        &compiled.arena,
        &UiThemeRegistry::default(),
    )
    .unwrap();
    let root = &resolved.nodes["root"];

    assert_eq!(root.self_values["background"].as_str(), Some("#3cc7d6"));
    assert_eq!(root.self_values["foreground"].as_str(), Some("#e8ecee"));
    assert_eq!(
        root.style_tokens.get("background").map(String::as_str),
        Some("theme.palette.accent")
    );
    assert_eq!(
        root.style_tokens.get("foreground").map(String::as_str),
        Some("theme.palette.text.primary")
    );
}

#[test]
fn ui_v2_surface_builder_resolves_theme_tokens_for_static_and_runtime_rules() {
    let mut document = v2_document("asset://ui/tests/theme_surface.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Button".to_string(),
            control_id: Some("ThemeSurfaceButton".to_string()),
            classes: vec!["primary".to_string()],
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "theme_surface_material".to_string(),
        rules: vec![
            style_rule(
                "Button.primary",
                [("background", "$theme.palette.surface.1")],
            ),
            style_rule(
                "Button.primary:hover",
                [("background", "$theme.palette.accent")],
            ),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document_with_theme(
        UiTreeId::new("runtime.ui.v2.theme_surface"),
        &document,
        &compiled,
        &UiThemeRegistry::default(),
    )
    .unwrap();
    let node_id = node_id_by_control_id(&surface, "ThemeSurfaceButton");

    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#171a1d")
    );
    assert_eq!(
        runtime_style_token(&surface, node_id, "background"),
        Some("theme.palette.surface.1")
    );

    assert!(surface.component_states.set_hovered(node_id, true));
    surface.mark_component_state_render_dirty(node_id).unwrap();
    assert_eq!(
        runtime_color_attr(&surface, node_id, "background"),
        Some("#3cc7d6")
    );
    assert_eq!(
        runtime_style_token(&surface, node_id, "background"),
        Some("theme.palette.accent")
    );
}

#[test]
fn ui_v2_surface_runtime_pseudo_state_restyles_from_retained_component_state() {
    let mut document = v2_document("asset://ui/tests/runtime_style.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Button".to_string(),
            control_id: Some("RuntimeButton".to_string()),
            classes: vec!["material".to_string()],
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_material".to_string(),
        rules: vec![
            style_rule("Button.material", [("background", "#101010")]),
            style_rule("Button.material:hover", [("background", "#202020")]),
            style_rule("Button.material:active", [("background", "#303030")]),
            style_rule("#RuntimeButton:focus", [("outline", "#404040")]),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_style"),
        &document,
        &compiled,
    )
    .unwrap();
    let node_id = surface.tree.roots[0];

    assert_eq!(
        surface
            .tree
            .nodes
            .get(&node_id)
            .unwrap()
            .template_metadata
            .as_ref()
            .unwrap()
            .attributes
            .get("background")
            .and_then(Value::as_str),
        Some("#101010")
    );
    assert!(surface
        .tree
        .nodes
        .get(&node_id)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes
        .get("outline")
        .is_none());

    assert!(surface.component_states.set_hovered(node_id, true));
    surface.mark_component_state_render_dirty(node_id).unwrap();
    assert_eq!(
        runtime_attr(&surface, node_id, "background"),
        Some("#202020")
    );
    let dirty = surface.tree.nodes.get(&node_id).unwrap().dirty;
    assert!(dirty.render);
    assert!(!dirty.style);
    assert!(!dirty.text);

    surface.clear_dirty_flags();
    assert!(surface.component_states.set_pressed(node_id, true));
    surface.mark_component_state_render_dirty(node_id).unwrap();
    assert_eq!(
        runtime_attr(&surface, node_id, "background"),
        Some("#303030")
    );

    surface.clear_dirty_flags();
    assert!(surface.component_states.set_focused(node_id, true));
    surface.mark_component_state_render_dirty(node_id).unwrap();
    assert_eq!(runtime_attr(&surface, node_id, "outline"), Some("#404040"));
    assert_eq!(
        runtime_attr(&surface, node_id, "background"),
        Some("#303030")
    );

    surface.clear_dirty_flags();
    assert!(surface.component_states.set_pressed(node_id, false));
    surface.mark_component_state_render_dirty(node_id).unwrap();
    assert_eq!(
        runtime_attr(&surface, node_id, "background"),
        Some("#202020")
    );

    surface.clear_dirty_flags();
    assert!(surface.component_states.set_hovered(node_id, false));
    surface.mark_component_state_render_dirty(node_id).unwrap();
    assert_eq!(
        runtime_attr(&surface, node_id, "background"),
        Some("#101010")
    );
    assert_eq!(runtime_attr(&surface, node_id, "outline"), Some("#404040"));
}

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
fn ui_v2_surface_authored_pseudo_state_is_seeded_but_not_baked_into_baseline() {
    let mut document = v2_document("asset://ui/tests/runtime_style_seeded.v2.ui", "root");
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "Button".to_string(),
            control_id: Some("SeededButton".to_string()),
            classes: vec!["material".to_string()],
            state: BTreeMap::from([("hovered".to_string(), Value::Boolean(true))]),
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_material".to_string(),
        rules: vec![
            style_rule("Button.material", [("background", "#101010")]),
            style_rule("Button.material:hover", [("background", "#202020")]),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_style_seeded"),
        &document,
        &compiled,
    )
    .unwrap();
    let node_id = surface.tree.roots[0];

    assert_eq!(
        runtime_attr(&surface, node_id, "background"),
        Some("#202020")
    );

    assert!(surface.component_states.set_hovered(node_id, false));
    surface.mark_component_state_render_dirty(node_id).unwrap();
    assert_eq!(
        runtime_attr(&surface, node_id, "background"),
        Some("#101010")
    );
}

#[test]
fn ui_v2_surface_runtime_pseudo_state_restyles_deep_descendant_from_parent_state() {
    const NODE_COUNT: usize = 512;

    let mut document = v2_document("asset://ui/tests/runtime_style_deep.v2.ui", "n0");
    for index in 0..NODE_COUNT {
        let is_leaf = index + 1 == NODE_COUNT;
        let child = (!is_leaf).then(|| UiV2ChildMount {
            node: format!("n{}", index + 1),
            slot: BTreeMap::new(),
        });
        document.nodes.insert(
            format!("n{index}"),
            UiV2NodeDefinition {
                component: if is_leaf {
                    "Text".to_string()
                } else {
                    "Container".to_string()
                },
                control_id: is_leaf.then(|| "DeepRuntimeLabel".to_string()),
                classes: if index == 0 {
                    vec!["runtime-host".to_string()]
                } else if is_leaf {
                    vec!["deep-label".to_string()]
                } else {
                    Vec::new()
                },
                children: child.into_iter().collect(),
                ..Default::default()
            },
        );
    }
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_deep_material".to_string(),
        rules: vec![
            style_rule("Text.deep-label", [("foreground", "#111111")]),
            style_rule(
                ".runtime-host:hover Text.deep-label",
                [("foreground", "#abcdef")],
            ),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.runtime_style_deep"),
        &document,
        &compiled,
    )
    .unwrap();
    let root_id = surface.tree.roots[0];
    let leaf_id = node_id_by_control_id(&surface, "DeepRuntimeLabel");

    assert_eq!(
        runtime_color_attr(&surface, leaf_id, "foreground"),
        Some("#111111")
    );

    assert!(surface.component_states.set_hovered(root_id, true));
    surface.mark_component_state_render_dirty(root_id).unwrap();
    assert_eq!(
        runtime_color_attr(&surface, leaf_id, "foreground"),
        Some("#abcdef")
    );
    let leaf_dirty = surface.tree.nodes.get(&leaf_id).unwrap().dirty;
    assert!(leaf_dirty.render);
    assert!(!leaf_dirty.layout);
    assert!(!leaf_dirty.style);

    surface.clear_dirty_flags();
    assert!(surface.component_states.set_hovered(root_id, false));
    surface.mark_component_state_render_dirty(root_id).unwrap();
    assert_eq!(
        runtime_color_attr(&surface, leaf_id, "foreground"),
        Some("#111111")
    );
}

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
