use super::*;
use std::collections::BTreeMap;

#[test]
fn tree_view_primary_click_selects_clicked_item_on_owner() {
    let mut surface = tree_view_pointer_route_surface(false);

    dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(24.0, 44.0),
        false,
        false,
    );
    let result = dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Up,
        UiPoint::new(24.0, 44.0),
        false,
        false,
    );

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::SelectOption {
                    property: "value".to_string(),
                    option_id: "Materials".to_string(),
                    selected: true,
                }
    }));
    assert_tree_attr_strings(&surface, "selected_items", &["Materials"]);
    assert_tree_attr_string(&surface, "value", "Materials");
    assert_tree_attr_int(&surface, "focused_index", 1);
    assert_tree_attr_int(&surface, "selected_index", 1);
    assert_tree_attr_int(&surface, "selection_anchor_index", 1);
}

#[test]
fn tree_view_control_click_toggles_item_in_multi_selection() {
    let mut surface = tree_view_pointer_route_surface(false);

    dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(24.0, 44.0),
        true,
        false,
    );
    let result = dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Up,
        UiPoint::new(24.0, 44.0),
        true,
        false,
    );

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::SelectOption {
                    property: "selected_items".to_string(),
                    option_id: "Materials".to_string(),
                    selected: true,
                }
    }));
    assert_tree_attr_strings(&surface, "selected_items", &["Assets", "Materials"]);
    assert_tree_attr_int(&surface, "selection_anchor_index", 1);

    dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(24.0, 44.0),
        true,
        false,
    );
    let toggled = dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Up,
        UiPoint::new(24.0, 44.0),
        true,
        false,
    );

    assert!(toggled.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::SelectOption {
                    property: "selected_items".to_string(),
                    option_id: "Materials".to_string(),
                    selected: false,
                }
    }));
    assert_tree_attr_strings(&surface, "selected_items", &["Assets"]);
    assert_tree_attr_int(&surface, "selection_anchor_index", 1);
}

#[test]
fn material_tree_view_shift_click_selects_anchor_to_target_range() {
    let mut surface = tree_view_pointer_route_surface(true);

    dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(24.0, 68.0),
        false,
        true,
    );
    let result = dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Up,
        UiPoint::new(24.0, 68.0),
        false,
        true,
    );

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::SelectOption {
                    property: "selectedItems".to_string(),
                    option_id: "Shaders".to_string(),
                    selected: true,
                }
    }));
    assert_tree_attr_strings(
        &surface,
        "selectedItems",
        &["Assets", "Materials", "Shaders"],
    );
    assert_tree_attr_int(&surface, "focused_index", 2);
    assert_tree_attr_int(&surface, "selected_index", 2);
    assert_tree_attr_int(&surface, "selectionAnchorIndex", 0);
}

#[test]
fn tree_view_double_click_begins_rename_for_clicked_item() {
    let mut surface = tree_view_pointer_route_surface(false);

    dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(24.0, 44.0),
        false,
        false,
    );
    let result = dispatch_tree_pointer_with_button(
        &mut surface,
        UiPointerEventKind::Up,
        UiPoint::new(24.0, 44.0),
        UiPointerButton::Primary,
        2,
        false,
        false,
    );

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::KeyboardAction {
                    action: UiComponentKeyboardAction::BeginEdit,
                }
    }));
    assert_tree_attr_bool(&surface, "editing", true);
    assert_tree_attr_string(&surface, "editing_node_id", "Materials");
    assert_tree_attr_string(&surface, "editing_text", "Materials");
    assert_tree_attr_int(&surface, "editing_index", 1);
    assert_tree_attr_bool(&surface, "rename_committed", false);
    assert_tree_attr_string(&surface, "renamed_node_id", "");
    assert_tree_attr_string(&surface, "renamed_text", "");
    assert_tree_attr_strings(&surface, "selected_items", &["Materials"]);
    assert_tree_attr_int(&surface, "focused_index", 1);
    assert_tree_attr_int(&surface, "selected_index", 1);
}

#[test]
fn material_tree_view_secondary_release_begins_context_rename_for_clicked_item() {
    let mut surface = tree_view_pointer_route_surface(true);

    dispatch_tree_pointer_with_button(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(24.0, 68.0),
        UiPointerButton::Secondary,
        1,
        false,
        false,
    );
    let result = dispatch_tree_pointer_with_button(
        &mut surface,
        UiPointerEventKind::Up,
        UiPoint::new(24.0, 68.0),
        UiPointerButton::Secondary,
        1,
        false,
        false,
    );

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::KeyboardAction {
                    action: UiComponentKeyboardAction::BeginEdit,
                }
    }));
    assert_tree_attr_bool(&surface, "editing", true);
    assert_tree_attr_string(&surface, "editingNodeId", "Shaders");
    assert_tree_attr_string(&surface, "editingText", "Shaders");
    assert_tree_attr_int(&surface, "editingIndex", 2);
    assert_tree_attr_bool(&surface, "renameCommitted", false);
    assert_tree_attr_string(&surface, "renamedNodeId", "");
    assert_tree_attr_string(&surface, "renamedText", "");
    assert_tree_attr_strings(&surface, "selectedItems", &["Assets"]);
    assert_tree_attr_int(&surface, "focused_index", 2);
    assert_tree_attr_int(&surface, "selected_index", 2);
}

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
fn tree_view_scroll_updates_virtual_window_and_emits_visible_range() {
    let mut surface = tree_view_virtualized_reparent_surface(false);

    let result = dispatch_tree_scroll(&mut surface, UiPoint::new(24.0, 76.0), 48.0);

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event == UiComponentEvent::SetVisibleRange { start: 2, count: 3 }
    }));
    assert_tree_attr_int(&surface, "total_count", 7);
    assert_tree_attr_int(&surface, "item_count", 7);
    assert_tree_attr_int(&surface, "viewport_start", 2);
    assert_tree_attr_int(&surface, "viewport_count", 3);
    assert_tree_attr_int(&surface, "visible_end", 5);
    assert_tree_attr_int(&surface, "visibleEnd", 5);
    assert_tree_attr_int(&surface, "requested_start", 1);
    assert_tree_attr_int(&surface, "requested_count", 5);
    assert_tree_attr_float(&surface, "scrollTop", 48.0);
}

#[test]
fn tree_view_virtualized_reparent_drag_updates_window() {
    let mut surface = tree_view_virtualized_reparent_surface(false);

    dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Down,
        UiPoint::new(24.0, 92.0),
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
    let result = dispatch_tree_pointer(
        &mut surface,
        UiPointerEventKind::Up,
        UiPoint::new(24.0, 44.0),
        false,
        false,
    );

    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event
                == UiComponentEvent::MoveElement {
                    property: "nodes".to_string(),
                    from: 3,
                    to: 2,
                }
    }));
    assert!(result.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && event.event == UiComponentEvent::SetVisibleRange { start: 2, count: 3 }
    }));
    assert_tree_node_order(&surface, "nodes", &["Assets", "Scenes"]);
    assert_tree_child_order(&surface, "nodes", "Materials", &["Shaders"]);
    assert_tree_child_order(&surface, "nodes", "Shaders", &["Vertex", "Fragment"]);
    assert_tree_attr_strings(&surface, "selected_items", &["Shaders"]);
    assert_tree_attr_strings(
        &surface,
        "expanded_items",
        &["Assets", "Shaders", "Materials"],
    );
    assert_tree_attr_int(&surface, "focused_index", 2);
    assert_tree_attr_int(&surface, "selected_index", 2);
    assert_tree_attr_int(&surface, "selection_anchor_index", 2);
    assert_tree_attr_int(&surface, "viewport_start", 2);
    assert_tree_attr_int(&surface, "visible_end", 5);
    assert_tree_attr_float(&surface, "scrollTop", 48.0);
}

fn dispatch_tree_pointer(
    surface: &mut UiSurface,
    kind: UiPointerEventKind,
    point: UiPoint,
    control: bool,
    shift: bool,
) -> UiInputDispatchResult {
    dispatch_tree_pointer_with_button(
        surface,
        kind,
        point,
        UiPointerButton::Primary,
        1,
        control,
        shift,
    )
}

fn dispatch_tree_pointer_with_button(
    surface: &mut UiSurface,
    kind: UiPointerEventKind,
    point: UiPoint,
    button: UiPointerButton,
    click_count: u8,
    control: bool,
    shift: bool,
) -> UiInputDispatchResult {
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            tree_pointer_event(kind, point, button, click_count, control, shift),
        )
        .expect("tree pointer event should dispatch")
}

fn dispatch_tree_scroll(
    surface: &mut UiSurface,
    point: UiPoint,
    scroll_delta: f32,
) -> UiInputDispatchResult {
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            tree_scroll_event(point, scroll_delta),
        )
        .expect("tree scroll event should dispatch")
}

fn tree_pointer_event(
    kind: UiPointerEventKind,
    point: UiPoint,
    button: UiPointerButton,
    click_count: u8,
    control: bool,
    shift: bool,
) -> UiInputEvent {
    let mut metadata = input_metadata();
    metadata.modifiers.control = control;
    metadata.modifiers.shift = shift;
    UiInputEvent::Pointer(UiPointerInputEvent {
        metadata,
        event: UiPointerEvent::new(kind, point)
            .with_button(button)
            .with_click_count(click_count),
        precise_scroll: None,
    })
}

fn tree_scroll_event(point: UiPoint, scroll_delta: f32) -> UiInputEvent {
    UiInputEvent::Pointer(UiPointerInputEvent {
        metadata: input_metadata(),
        event: UiPointerEvent::new(UiPointerEventKind::Scroll, point)
            .with_scroll_delta(scroll_delta),
        precise_scroll: None,
    })
}

fn tree_view_pointer_route_surface(material: bool) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.input.reply_route.tree_view"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 180.0, 120.0))
            .with_state_flags(input_state()),
    );

    let (component, selected_property, anchor_property) = if material {
        ("MaterialTreeView", "selectedItems", "selectionAnchorIndex")
    } else {
        ("TreeView", "selected_items", "selection_anchor_index")
    };
    let node_property = if material { "items" } else { "nodes" };
    let reorder_property = if material {
        "itemsReordering = true"
    } else {
        "reorderable = true"
    };
    let mut tree_attributes: BTreeMap<String, toml::Value> = toml::from_str(&format!(
        r#"
{node_property} = [
    {{ id = "Assets", label = "Assets" }},
    {{ id = "Materials", label = "Materials" }},
    {{ id = "Shaders", label = "Shaders" }},
]
{selected_property} = ["Assets"]
{anchor_property} = 0
multi_select = true
multiSelect = true
{reorder_property}
"#
    ))
    .expect("tree attributes should parse");

    if material {
        tree_attributes.remove("selected_items");
    }

    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/tree"))
                .with_frame(UiFrame::new(10.0, 10.0, 140.0, 90.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: component.to_string(),
                    control_id: Some("SceneTree".to_string()),
                    attributes: tree_attributes,
                    bindings: vec![
                        binding("SceneTree/SelectOption", UiEventKind::Change),
                        binding("SceneTree/KeyboardAction", UiEventKind::Change),
                        binding("SceneTree/BeginDrag", UiEventKind::DragBegin),
                        binding("SceneTree/EndDrag", UiEventKind::DragEnd),
                        binding("SceneTree/MoveElement", UiEventKind::Change),
                    ],
                    ..Default::default()
                }),
        )
        .unwrap();

    for (index, item_id) in ["Assets", "Materials", "Shaders"].into_iter().enumerate() {
        surface
            .tree
            .insert_child(
                UiNodeId::new(2),
                UiTreeNode::new(
                    UiNodeId::new(3 + index as u64),
                    UiNodePath::new(format!("root/tree/{item_id}")),
                )
                .with_frame(UiFrame::new(18.0, 18.0 + index as f32 * 24.0, 120.0, 20.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "TreeItem".to_string(),
                    control_id: Some(format!("{item_id}Row")),
                    attributes: toml::from_str(&format!(r#"itemId = "{item_id}""#))
                        .expect("tree item attributes should parse"),
                    ..Default::default()
                }),
            )
            .unwrap();
    }

    surface.rebuild();
    surface
}

fn tree_view_virtualized_reparent_surface(material: bool) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new(
        "runtime.ui.input.reply_route.tree_view.virtualized_reparent",
    ));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 220.0, 220.0))
            .with_state_flags(input_state()),
    );

    let (component, node_property, selected_property, anchor_property, reorder_property) =
        if material {
            (
                "MaterialTreeView",
                "items",
                "selectedItems",
                "selectionAnchorIndex",
                "itemsReordering = true",
            )
        } else {
            (
                "TreeView",
                "nodes",
                "selected_items",
                "selection_anchor_index",
                "reorderable = true",
            )
        };
    let mut tree_attributes: BTreeMap<String, toml::Value> = toml::from_str(&format!(
        r#"
{node_property} = [
    {{ id = "Assets", label = "Assets", children = [
        {{ id = "Materials", label = "Materials" }},
        {{ id = "Textures", label = "Textures" }},
    ] }},
    {{ id = "Shaders", label = "Shaders", children = [
        {{ id = "Vertex", label = "Vertex" }},
        {{ id = "Fragment", label = "Fragment" }},
    ] }},
    {{ id = "Scenes", label = "Scenes" }},
]
{selected_property} = ["Assets"]
{anchor_property} = 0
expanded_items = ["Assets", "Shaders"]
multi_select = true
multiSelect = true
{reorder_property}
rowHeight = 24.0
overscanCount = 1
viewport_start = 0
viewport_count = 3
scrollTop = 0.0
"#
    ))
    .expect("virtualized tree attributes should parse");

    if material {
        tree_attributes.remove("selected_items");
    }

    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/tree"))
                .with_frame(UiFrame::new(10.0, 10.0, 170.0, 170.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: component.to_string(),
                    control_id: Some("VirtualSceneTree".to_string()),
                    attributes: tree_attributes,
                    bindings: vec![
                        binding("VirtualSceneTree/SelectOption", UiEventKind::Change),
                        binding("VirtualSceneTree/SetVisibleRange", UiEventKind::Change),
                        binding("VirtualSceneTree/BeginDrag", UiEventKind::DragBegin),
                        binding("VirtualSceneTree/EndDrag", UiEventKind::DragEnd),
                        binding("VirtualSceneTree/MoveElement", UiEventKind::Change),
                    ],
                    ..Default::default()
                }),
        )
        .unwrap();

    for (index, item_id) in [
        "Assets",
        "Materials",
        "Textures",
        "Shaders",
        "Vertex",
        "Fragment",
        "Scenes",
    ]
    .into_iter()
    .enumerate()
    {
        surface
            .tree
            .insert_child(
                UiNodeId::new(2),
                UiTreeNode::new(
                    UiNodeId::new(20 + index as u64),
                    UiNodePath::new(format!("root/tree/{item_id}")),
                )
                .with_frame(UiFrame::new(18.0, 18.0 + index as f32 * 24.0, 140.0, 20.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "TreeItem".to_string(),
                    control_id: Some(format!("{item_id}Row")),
                    attributes: toml::from_str(&format!(r#"itemId = "{item_id}""#))
                        .expect("tree item attributes should parse"),
                    ..Default::default()
                }),
            )
            .unwrap();
    }

    surface.rebuild();
    surface
}

fn assert_tree_attr_strings(surface: &UiSurface, property: &str, expected: &[&str]) {
    let actual = tree_attr(surface, property)
        .and_then(toml::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(toml::Value::as_str)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    assert_eq!(actual, expected);
}

fn assert_tree_attr_string(surface: &UiSurface, property: &str, expected: &str) {
    assert_eq!(
        tree_attr(surface, property).and_then(toml::Value::as_str),
        Some(expected)
    );
}

fn assert_tree_attr_bool(surface: &UiSurface, property: &str, expected: bool) {
    assert_eq!(
        tree_attr(surface, property).and_then(toml::Value::as_bool),
        Some(expected)
    );
}

fn assert_tree_attr_float(surface: &UiSurface, property: &str, expected: f64) {
    assert_eq!(
        tree_attr(surface, property).and_then(toml_number),
        Some(expected)
    );
}

fn toml_number(value: &toml::Value) -> Option<f64> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|value| value as f64))
}

fn assert_tree_attr_int(surface: &UiSurface, property: &str, expected: i64) {
    assert_eq!(
        tree_attr(surface, property).and_then(toml::Value::as_integer),
        Some(expected)
    );
}

fn assert_tree_node_order(surface: &UiSurface, property: &str, expected: &[&str]) {
    let actual = tree_attr(surface, property)
        .and_then(toml::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(|value| {
                    value
                        .as_table()
                        .and_then(|table| table.get("id"))
                        .and_then(toml::Value::as_str)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    assert_eq!(actual, expected);
}

fn assert_tree_child_order(
    surface: &UiSurface,
    property: &str,
    parent_id: &str,
    expected: &[&str],
) {
    let actual = tree_attr(surface, property)
        .and_then(|value| find_tree_node(value, parent_id))
        .and_then(|node| node.as_table())
        .and_then(|node| node.get("children"))
        .and_then(toml::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(|value| {
                    value
                        .as_table()
                        .and_then(|table| table.get("id"))
                        .and_then(toml::Value::as_str)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    assert_eq!(actual, expected);
}

fn tree_attr<'a>(surface: &'a UiSurface, property: &str) -> Option<&'a toml::Value> {
    surface
        .tree
        .node(UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get(property))
}

fn find_tree_node<'a>(value: &'a toml::Value, node_id: &str) -> Option<&'a toml::Value> {
    match value {
        toml::Value::Array(values) => values
            .iter()
            .find_map(|value| find_tree_node(value, node_id)),
        toml::Value::Table(values) => {
            if values
                .get("id")
                .and_then(toml::Value::as_str)
                .is_some_and(|id| id == node_id)
            {
                return Some(value);
            }
            values
                .get("children")
                .and_then(|children| find_tree_node(children, node_id))
        }
        _ => None,
    }
}
