use super::super::*;

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
