//! 拖拽释放改变树节点顺序并发移动事件；材料组件的 items 别名应得到相同结果。

use super::*;

#[test]
fn tree_view_drag_release_reorders_nodes_and_emits_move_element() {
    let mut surface = tree_view_pointer_route_surface(false);

    let begin = dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(24.0, 44.0),
        false,
        false,
    );
    assert!(begin.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::BeginDrag {
                    property: "nodes".to_string(),
                }
            && event
                .drag
                .as_ref()
                .is_some_and(|drag| drag.phase == UiDragPhase::Begin)
    }));

    dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Move,
        UiPoint::new(24.0, 68.0),
        false,
        false,
    );
    let result = dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Up,
        UiPoint::new(24.0, 68.0),
        false,
        false,
    );

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::MoveElement {
                    property: "nodes".to_string(),
                    from: 1,
                    to: 2,
                }
    }));
    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::EndDrag {
                    property: "nodes".to_string(),
                }
            && event
                .drag
                .as_ref()
                .is_some_and(|drag| drag.phase == UiDragPhase::End)
    }));
    assert_tree_node_order(&surface, "nodes", &["Assets", "Shaders", "Materials"]);
    assert_tree_attr_strings(&surface, "selected_items", &["Materials"]);
    assert_tree_attr_int(&surface, "focused_index", 2);
    assert_tree_attr_int(&surface, "selected_index", 2);
}

#[test]
fn material_tree_view_items_reordering_reorders_items_array() {
    let mut surface = tree_view_pointer_route_surface(true);

    dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(24.0, 68.0),
        false,
        false,
    );
    dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Move,
        UiPoint::new(24.0, 20.0),
        false,
        false,
    );
    let result = dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Up,
        UiPoint::new(24.0, 20.0),
        false,
        false,
    );

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::MoveElement {
                    property: "items".to_string(),
                    from: 2,
                    to: 0,
                }
    }));
    assert_tree_node_order(&surface, "items", &["Shaders", "Assets", "Materials"]);
    assert_tree_attr_strings(&surface, "selectedItems", &["Shaders"]);
    assert_tree_attr_int(&surface, "focused_index", 0);
    assert_tree_attr_int(&surface, "selected_index", 0);
}

#[test]
fn same_preorder_position_reparent_commits_the_model_selection_and_virtual_window() {
    for material in [false, true] {
        let mut surface = tree_view_pointer_route_surface(material);
        let property = if material { "items" } else { "nodes" };
        let selected = if material {
            "selectedItems"
        } else {
            "selected_items"
        };
        let anchor = if material {
            "selectionAnchorIndex"
        } else {
            "selection_anchor_index"
        };
        let expanded = if material {
            "expandedItems"
        } else {
            "expanded_items"
        };
        let mut nested: BTreeMap<String, toml::Value> = toml::from_str(
            r#"model = [
                { id = "Assets", label = "Assets", children = [
                    { id = "Materials", label = "Materials" }
                ] },
                { id = "Shaders", label = "Shaders" }
            ]"#,
        )
        .unwrap();
        let attributes = &mut surface
            .tree
            .node_mut(UiNodeId::new(2))
            .unwrap()
            .template_metadata
            .as_mut()
            .unwrap()
            .attributes;
        attributes.insert(property.to_owned(), nested.remove("model").unwrap());
        attributes.insert(
            expanded.to_owned(),
            toml::Value::Array(vec![toml::Value::String("Assets".to_owned())]),
        );
        attributes.insert("viewport_start".to_owned(), toml::Value::Integer(0));
        attributes.insert("viewport_count".to_owned(), toml::Value::Integer(1));
        attributes.insert("overscan".to_owned(), toml::Value::Integer(0));

        for kind in [UiPointerEventKind::Down, UiPointerEventKind::Move] {
            dispatch_tree_pointer(
                &mut surface,
                kind,
                if kind == UiPointerEventKind::Down {
                    UiPoint::new(24.0, 68.0)
                } else {
                    UiPoint::new(24.0, 44.0)
                },
                false,
                false,
            );
        }
        let result = dispatch_tree_pointer(
            &mut surface,
            UiPointerEventKind::Up,
            UiPoint::new(24.0, 44.0),
            false,
            false,
        );

        assert_tree_node_order(&surface, property, &["Assets"]);
        assert_tree_child_order(&surface, property, "Assets", &["Materials"]);
        assert_tree_child_order(&surface, property, "Materials", &["Shaders"]);
        assert_tree_attr_strings(&surface, selected, &["Shaders"]);
        assert_tree_attr_strings(&surface, expanded, &["Assets", "Materials"]);
        assert_tree_attr_int(&surface, "focused_index", 2);
        assert_tree_attr_int(&surface, "selected_index", 2);
        assert_tree_attr_int(&surface, anchor, 2);
        assert_tree_attr_int(&surface, "viewport_start", 2);
        assert_tree_attr_int(&surface, "requested_start", 2);
        assert_tree_attr_int(&surface, "requested_count", 1);
        assert!(!result.binding_reports.is_empty());
        assert!(result.component_events.iter().any(|event| {
            event.target == UiNodeId::new(2)
                && event.event
                    == UiComponentEvent::MoveElement {
                        property: property.to_owned(),
                        from: 2,
                        to: 2,
                    }
        }));
        assert!(result.component_events.iter().any(|event| {
            event.event == UiComponentEvent::SetVisibleRange { start: 2, count: 1 }
        }));
        assert!(result.component_events.iter().any(|event| {
            event.event
                == UiComponentEvent::EndDrag {
                    property: property.to_owned(),
                }
        }));

        // Removing and reinserting the same final child under the same parent is a no-op.
        dispatch_tree_pointer(
            &mut surface,
            UiPointerEventKind::Down,
            UiPoint::new(24.0, 68.0),
            false,
            false,
        );
        dispatch_tree_pointer(
            &mut surface,
            UiPointerEventKind::Move,
            UiPoint::new(24.0, 44.0),
            false,
            false,
        );
        let unchanged = dispatch_tree_pointer(
            &mut surface,
            UiPointerEventKind::Up,
            UiPoint::new(24.0, 44.0),
            false,
            false,
        );
        assert!(!unchanged
            .component_events
            .iter()
            .any(|event| { matches!(event.event, UiComponentEvent::MoveElement { .. }) }));
        assert!(unchanged.component_events.iter().any(|event| {
            event.event
                == UiComponentEvent::EndDrag {
                    property: property.to_owned(),
                }
        }));
        assert_tree_child_order(&surface, property, "Materials", &["Shaders"]);
    }
}
