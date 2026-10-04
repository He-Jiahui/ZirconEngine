use super::super::*;

#[test]
fn ui_v2_runtime_style_attribute_delta_retains_one_menu_item() {
    assert_runtime_style_retains_menu_payload(1);
}

#[test]
fn ui_v2_runtime_style_attribute_delta_retains_one_hundred_menu_items() {
    assert_runtime_style_retains_menu_payload(100);
}

#[test]
fn ui_v2_runtime_style_attribute_delta_retains_ten_thousand_menu_items() {
    assert_runtime_style_retains_menu_payload(10_000);
}

fn assert_runtime_style_retains_menu_payload(item_count: usize) {
    let mut document = v2_document("asset://ui/tests/runtime_style_payload.zui", "root");
    let items = (0..item_count)
        .map(|index| {
            Value::Table(toml::map::Map::from_iter([
                (
                    "id".to_string(),
                    Value::String(format!("retained-item-{index}")),
                ),
                (
                    "label".to_string(),
                    Value::String(format!(
                        "Menu item {index:05}: retained nested label payload"
                    )),
                ),
            ]))
        })
        .collect();
    document.nodes.insert(
        "root".to_string(),
        UiV2NodeDefinition {
            component: "ContextActionMenu".to_string(),
            control_id: Some("RetainedPayloadMenu".to_string()),
            classes: vec!["retained-payload".to_string()],
            props: BTreeMap::from([("menu_items".to_string(), Value::Array(items))]),
            layout: Some(fixed_size_layout(240.0, 80.0)),
            ..Default::default()
        },
    );
    document.stylesheets.push(UiV2StyleSheet {
        id: "runtime_payload".to_string(),
        rules: vec![
            style_rule(
                "ContextActionMenu.retained-payload",
                [("background", "#101010")],
            ),
            style_rule(
                "ContextActionMenu.retained-payload:hover",
                [("background", "#202020"), ("outline", "#404040")],
            ),
        ],
    });

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new(format!("runtime.ui.v2.style_payload.{item_count}")),
        &document,
        &compiled,
    )
    .unwrap();
    let node_id = node_id_by_control_id(&surface, "RetainedPayloadMenu");
    surface.compute_layout(UiSize::new(240.0, 80.0)).unwrap();
    surface.rebuild();
    surface.clear_dirty_flags();

    let original_addresses = menu_payload_addresses(&surface, node_id, item_count);
    assert_eq!(
        runtime_attr(&surface, node_id, "background"),
        Some("#101010")
    );
    assert_eq!(runtime_attr(&surface, node_id, "outline"), None);
    assert_unchanged_restyle_retains_menu_payload(
        &mut surface,
        node_id,
        item_count,
        original_addresses,
    );

    // A projection-only restyle changes metadata without marking retained dirty state.
    assert!(surface.component_states.set_hovered(node_id, true));
    assert_eq!(
        surface
            .apply_runtime_state_style_node(node_id, false)
            .unwrap(),
        1
    );
    assert_eq!(
        runtime_attr(&surface, node_id, "background"),
        Some("#202020")
    );
    assert!(!surface.tree.nodes.get(&node_id).unwrap().dirty.any());
    assert_eq!(
        menu_payload_addresses(&surface, node_id, item_count),
        original_addresses
    );
    assert!(surface.component_states.set_hovered(node_id, false));
    assert_eq!(
        surface
            .apply_runtime_state_style_node(node_id, false)
            .unwrap(),
        1
    );
    assert_eq!(
        runtime_attr(&surface, node_id, "background"),
        Some("#101010")
    );
    assert!(!surface.tree.nodes.get(&node_id).unwrap().dirty.any());
    assert_eq!(
        menu_payload_addresses(&surface, node_id, item_count),
        original_addresses
    );

    for _ in 0..2 {
        assert!(surface.component_states.set_hovered(node_id, true));
        surface.mark_component_state_render_dirty(node_id).unwrap();
        assert_eq!(
            runtime_attr(&surface, node_id, "background"),
            Some("#202020")
        );
        assert_eq!(runtime_attr(&surface, node_id, "outline"), Some("#404040"));
        assert_eq!(
            menu_payload_addresses(&surface, node_id, item_count),
            original_addresses,
            "hover must retain the {item_count}-item array and nested label allocations",
        );
        let dirty = surface.tree.nodes.get(&node_id).unwrap().dirty;
        assert!(dirty.render);
        assert!(!dirty.style && !dirty.text && !dirty.layout);
        surface.clear_dirty_flags();
        assert_unchanged_restyle_retains_menu_payload(
            &mut surface,
            node_id,
            item_count,
            original_addresses,
        );

        assert!(surface.component_states.set_hovered(node_id, false));
        surface.mark_component_state_render_dirty(node_id).unwrap();
        assert_eq!(
            runtime_attr(&surface, node_id, "background"),
            Some("#101010")
        );
        assert_eq!(runtime_attr(&surface, node_id, "outline"), None);
        assert!(!surface
            .tree
            .nodes
            .get(&node_id)
            .unwrap()
            .template_metadata
            .as_ref()
            .unwrap()
            .style_overrides
            .contains_key("outline"));
        assert_eq!(
            menu_payload_addresses(&surface, node_id, item_count),
            original_addresses,
            "unmatching hover must restore style without replacing the {item_count}-item payload",
        );
        surface.clear_dirty_flags();
        assert_unchanged_restyle_retains_menu_payload(
            &mut surface,
            node_id,
            item_count,
            original_addresses,
        );
    }
}

fn assert_unchanged_restyle_retains_menu_payload(
    surface: &mut UiSurface,
    node_id: UiNodeId,
    item_count: usize,
    original_addresses: (*const Value, [*const u8; 3]),
) {
    assert_eq!(
        surface
            .apply_runtime_state_style_node(node_id, true)
            .unwrap(),
        0
    );
    assert!(!surface.tree.nodes.get(&node_id).unwrap().dirty.any());
    assert_eq!(
        menu_payload_addresses(surface, node_id, item_count),
        original_addresses,
        "unchanged restyle must retain the {item_count}-item payload",
    );
}

fn menu_payload_addresses(
    surface: &UiSurface,
    node_id: UiNodeId,
    item_count: usize,
) -> (*const Value, [*const u8; 3]) {
    let items = surface
        .tree
        .nodes
        .get(&node_id)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes
        .get("menu_items")
        .and_then(Value::as_array)
        .unwrap();
    assert_eq!(items.len(), item_count);
    let label_addresses = [0, item_count / 2, item_count - 1].map(|index| {
        let label = items[index]
            .as_table()
            .unwrap()
            .get("label")
            .and_then(Value::as_str)
            .unwrap();
        assert_eq!(
            label,
            format!("Menu item {index:05}: retained nested label payload"),
        );
        label.as_ptr()
    });
    // These pointers are compared only; no stale address is ever dereferenced.
    (items.as_ptr(), label_addresses)
}
