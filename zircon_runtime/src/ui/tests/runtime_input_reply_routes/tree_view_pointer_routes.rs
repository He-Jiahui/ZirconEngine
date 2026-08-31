use super::*;
use std::collections::BTreeMap;

mod drag_reorder;
mod selection;
mod virtualization;

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
    tree_attributes.insert(
        "component_role".to_string(),
        toml::Value::String(if material {
            "mui-x-tree-view".to_string()
        } else {
            "tree-view".to_string()
        }),
    );

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
                    attributes: toml::from_str(&format!(
                        r#"
itemId = "{item_id}"
component_role = "tree-item"
"#
                    ))
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
    tree_attributes.insert(
        "component_role".to_string(),
        toml::Value::String(if material {
            "mui-x-tree-view".to_string()
        } else {
            "tree-view".to_string()
        }),
    );

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
                    attributes: toml::from_str(&format!(
                        r#"
itemId = "{item_id}"
component_role = "tree-item"
"#
                    ))
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
