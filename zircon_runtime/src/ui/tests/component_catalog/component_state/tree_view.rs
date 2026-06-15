use std::collections::BTreeMap;

use crate::ui::component::{UiComponentDescriptorRegistry, UiComponentStateRuntimeExt};
use zircon_runtime_interface::ui::component::{
    UiComponentEvent, UiComponentEventKind, UiComponentKeyboardAction, UiComponentState, UiValue,
};

#[test]
fn tree_view_keyboard_expand_and_collapse_updates_focused_node_ids() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let tree = registry
        .descriptor("TreeView")
        .expect("TreeView descriptor");
    assert!(tree.supports_event(UiComponentEventKind::KeyboardAction));
    assert!(tree.supports_event(UiComponentEventKind::ToggleExpanded));
    assert!(tree.prop("expanded_items").is_some());

    let mut state = UiComponentState::new()
        .with_value(
            "nodes",
            UiValue::Array(vec![
                tree_node("Assets", vec![tree_node("Materials", Vec::new())]),
                tree_node("Scenes", Vec::new()),
            ]),
        )
        .with_value("focused_index", UiValue::Int(1))
        .with_value(
            "expanded_items",
            UiValue::Array(vec![UiValue::String("Assets".to_string())]),
        );

    state
        .apply_event(
            tree,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Increment,
            },
        )
        .unwrap();
    assert_eq!(
        state.value("expanded_items"),
        Some(&UiValue::Array(vec![
            UiValue::String("Assets".to_string()),
            UiValue::String("Materials".to_string()),
        ]))
    );
    assert_eq!(state.value("expanded"), Some(&UiValue::Bool(true)));
    assert!(state.flags.expanded);

    state
        .apply_event(
            tree,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Increment,
            },
        )
        .unwrap();
    assert_eq!(
        state.value("expanded_items"),
        Some(&UiValue::Array(vec![
            UiValue::String("Assets".to_string()),
            UiValue::String("Materials".to_string()),
        ])),
        "repeated expand should not duplicate node ids"
    );

    state
        .apply_event(
            tree,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Decrement,
            },
        )
        .unwrap();
    assert_eq!(
        state.value("expanded_items"),
        Some(&UiValue::Array(vec![UiValue::String("Assets".to_string())]))
    );
    assert!(state.flags.expanded);

    state = state.with_value("focused_index", UiValue::Int(0));
    state
        .apply_event(tree, UiComponentEvent::ToggleExpanded { expanded: false })
        .unwrap();
    assert_eq!(state.value("expanded_items"), Some(&UiValue::Array(vec![])));
    assert_eq!(state.value("expanded"), Some(&UiValue::Bool(false)));
    assert!(!state.flags.expanded);
}

#[test]
fn material_tree_view_writes_controlled_expanded_items_from_default_seed() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let tree = registry
        .descriptor("MaterialTreeView")
        .expect("MUI X TreeView descriptor");
    assert!(tree.prop("expandedItems").is_some());
    assert!(tree.prop("defaultExpandedItems").is_some());

    let mut state = UiComponentState::new()
        .with_value(
            "items",
            UiValue::Array(vec![
                tree_node("Assets", vec![tree_node("Materials", Vec::new())]),
                tree_node("Scenes", Vec::new()),
            ]),
        )
        .with_value("focused_index", UiValue::Int(1))
        .with_value(
            "defaultExpandedItems",
            UiValue::Array(vec![UiValue::String("Assets".to_string())]),
        );

    state
        .apply_event(
            tree,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Increment,
            },
        )
        .unwrap();

    assert_eq!(
        state.value("expandedItems"),
        Some(&UiValue::Array(vec![
            UiValue::String("Assets".to_string()),
            UiValue::String("Materials".to_string()),
        ]))
    );
    assert_eq!(
        state.value("defaultExpandedItems"),
        Some(&UiValue::Array(vec![UiValue::String("Assets".to_string())])),
        "controlled expandedItems should be written without mutating defaultExpandedItems"
    );
}

#[test]
fn tree_view_select_option_toggles_multi_selected_items_and_anchor() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let tree = registry
        .descriptor("TreeView")
        .expect("TreeView descriptor");
    assert!(tree.supports_event(UiComponentEventKind::SelectOption));
    assert!(tree.prop("selected_items").is_some());
    assert!(tree.prop("multi_select").is_some());
    assert!(tree.prop("selection_anchor_index").is_some());

    let mut state = UiComponentState::new()
        .with_value(
            "nodes",
            UiValue::Array(vec![
                tree_node("Assets", vec![tree_node("Materials", Vec::new())]),
                tree_node("Scenes", Vec::new()),
            ]),
        )
        .with_value("multi_select", UiValue::Bool(true))
        .with_value(
            "selected_items",
            UiValue::Array(vec![UiValue::String("Assets".to_string())]),
        );

    state
        .apply_event(
            tree,
            UiComponentEvent::SelectOption {
                property: "selected_items".to_string(),
                option_id: "Materials".to_string(),
                selected: true,
            },
        )
        .unwrap();

    assert_eq!(
        state.value("selected_items"),
        Some(&UiValue::Array(vec![
            UiValue::String("Assets".to_string()),
            UiValue::String("Materials".to_string()),
        ]))
    );
    assert_eq!(state.value("focused_index"), Some(&UiValue::Int(1)));
    assert_eq!(state.value("selected_index"), Some(&UiValue::Int(1)));
    assert_eq!(
        state.value("selection_anchor_index"),
        Some(&UiValue::Int(1))
    );
    assert!(state.flags.selected);

    state
        .apply_event(
            tree,
            UiComponentEvent::SelectOption {
                property: "selected_items".to_string(),
                option_id: "Assets".to_string(),
                selected: false,
            },
        )
        .unwrap();

    assert_eq!(
        state.value("selected_items"),
        Some(&UiValue::Array(vec![UiValue::String(
            "Materials".to_string()
        )]))
    );
    assert_eq!(
        state.value("selection_anchor_index"),
        Some(&UiValue::Int(0))
    );
    assert!(state.flags.selected);
}

#[test]
fn material_tree_view_range_selection_writes_controlled_selected_items() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let tree = registry
        .descriptor("MaterialTreeView")
        .expect("MUI X TreeView descriptor");
    assert!(tree.prop("selectedItems").is_some());
    assert!(tree.prop("multiSelect").is_some());
    assert!(tree.prop("range_selecting").is_some());
    assert!(tree.prop("rangeSelecting").is_some());
    assert!(tree.prop("selectionAnchorIndex").is_some());

    let mut state = UiComponentState::new()
        .with_value(
            "items",
            UiValue::Array(vec![
                tree_node("Assets", vec![tree_node("Materials", Vec::new())]),
                tree_node("Shaders", Vec::new()),
                tree_node("Scenes", Vec::new()),
            ]),
        )
        .with_value("multiSelect", UiValue::Bool(true))
        .with_value("range_selecting", UiValue::Bool(true))
        .with_value("selection_anchor_index", UiValue::Int(0))
        .with_value(
            "selectedItems",
            UiValue::Array(vec![UiValue::String("Scenes".to_string())]),
        );

    state
        .apply_event(
            tree,
            UiComponentEvent::SelectOption {
                property: "selectedItems".to_string(),
                option_id: "Shaders".to_string(),
                selected: true,
            },
        )
        .unwrap();

    assert_eq!(
        state.value("selectedItems"),
        Some(&UiValue::Array(vec![
            UiValue::String("Assets".to_string()),
            UiValue::String("Materials".to_string()),
            UiValue::String("Shaders".to_string()),
        ]))
    );
    assert_eq!(state.value("focused_index"), Some(&UiValue::Int(2)));
    assert_eq!(state.value("selected_index"), Some(&UiValue::Int(2)));
    assert_eq!(
        state.value("selection_anchor_index"),
        Some(&UiValue::Int(0))
    );
    assert!(state.flags.selected);
}

#[test]
fn tree_view_f2_begins_rename_escape_cancels() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let tree = registry
        .descriptor("TreeView")
        .expect("TreeView descriptor");
    assert!(tree.supports_event(UiComponentEventKind::KeyboardAction));
    assert!(tree.prop("editable").is_some());
    assert!(tree.prop("editing_node_id").is_some());
    assert!(tree.prop("editing_text").is_some());
    assert!(tree.prop("editing_index").is_some());

    let mut state = UiComponentState::new()
        .with_value(
            "nodes",
            UiValue::Array(vec![
                tree_node("Assets", vec![tree_node("Materials", Vec::new())]),
                tree_node("Scenes", Vec::new()),
            ]),
        )
        .with_value("focused_index", UiValue::Int(1));

    state
        .apply_event(
            tree,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::BeginEdit,
            },
        )
        .unwrap();

    assert_eq!(state.value("editing"), Some(&UiValue::Bool(true)));
    assert_eq!(
        state.value("editing_node_id"),
        Some(&UiValue::String("Materials".to_string()))
    );
    assert_eq!(state.value("editing_index"), Some(&UiValue::Int(1)));
    assert_eq!(
        state.value("editing_text"),
        Some(&UiValue::String("Materials".to_string()))
    );
    assert!(state.flags.focused);

    state
        .apply_event(
            tree,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::Cancel,
            },
        )
        .unwrap();

    assert_eq!(state.value("editing"), Some(&UiValue::Bool(false)));
    assert_eq!(
        state.value("editing_node_id"),
        Some(&UiValue::String(String::new()))
    );
    assert_eq!(state.value("editing_index"), Some(&UiValue::Int(-1)));
    assert_eq!(
        state.value("editing_text"),
        Some(&UiValue::String(String::new()))
    );
}

#[test]
fn tree_view_commit_rename_records_semantic_payload() {
    let registry = UiComponentDescriptorRegistry::material_editor_foundation();
    let tree = registry
        .descriptor("TreeView")
        .expect("TreeView descriptor");
    assert!(tree.supports_event(UiComponentEventKind::Commit));
    assert!(tree.prop("renamed_node_id").is_some());
    assert!(tree.prop("renamed_text").is_some());
    assert!(tree.prop("rename_committed").is_some());

    let mut state = UiComponentState::new()
        .with_value(
            "nodes",
            UiValue::Array(vec![tree_node(
                "Assets",
                vec![tree_node("Materials", Vec::new())],
            )]),
        )
        .with_value("focused_index", UiValue::Int(1));

    state
        .apply_event(
            tree,
            UiComponentEvent::KeyboardAction {
                action: UiComponentKeyboardAction::BeginEdit,
            },
        )
        .unwrap();
    state
        .apply_event(
            tree,
            UiComponentEvent::Commit {
                property: "editing_text".to_string(),
                value: UiValue::String("Surface Materials".to_string()),
            },
        )
        .unwrap();

    assert_eq!(state.value("editing"), Some(&UiValue::Bool(false)));
    assert_eq!(
        state.value("editing_node_id"),
        Some(&UiValue::String(String::new()))
    );
    assert_eq!(
        state.value("renamed_node_id"),
        Some(&UiValue::String("Materials".to_string()))
    );
    assert_eq!(
        state.value("renamed_text"),
        Some(&UiValue::String("Surface Materials".to_string()))
    );
    assert_eq!(state.value("rename_committed"), Some(&UiValue::Bool(true)));
}

fn tree_node(id: &str, children: Vec<UiValue>) -> UiValue {
    let mut node = BTreeMap::new();
    node.insert("id".to_string(), UiValue::String(id.to_string()));
    node.insert("label".to_string(), UiValue::String(id.to_string()));
    if !children.is_empty() {
        node.insert("children".to_string(), UiValue::Array(children));
    }
    UiValue::Map(node)
}
