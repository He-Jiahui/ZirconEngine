use super::*;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodePath, UiTreeId},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[test]
fn virtual_window_numeric_property_change_is_not_a_runtime_pseudo_state_change() {
    let mut states = UiSurfaceComponentStateStore::default();
    let node_id = UiNodeId::new(7);

    let change = states.sync_from_property(node_id, "viewport_start", &UiValue::Int(12));

    assert_eq!(
        change,
        UiComponentStatePropertyChange {
            value_changed: true,
            pseudo_state_changed: false,
        }
    );
    assert_eq!(
        states
            .get(node_id)
            .and_then(|state| state.value("viewport_start")),
        Some(&UiValue::Int(12))
    );
}

#[test]
fn virtual_window_pseudo_state_change_reports_only_real_flag_transitions() {
    let mut states = UiSurfaceComponentStateStore::default();
    let node_id = UiNodeId::new(9);

    let first = states.sync_from_property(node_id, "hovered", &UiValue::Bool(true));
    let alias = states.sync_from_property(node_id, "hover", &UiValue::Bool(true));
    let unchanged = states.sync_from_property(node_id, "hover", &UiValue::Bool(true));

    assert_eq!(
        first,
        UiComponentStatePropertyChange {
            value_changed: true,
            pseudo_state_changed: true,
        }
    );
    assert_eq!(
        alias,
        UiComponentStatePropertyChange {
            value_changed: true,
            pseudo_state_changed: false,
        }
    );
    assert_eq!(unchanged, UiComponentStatePropertyChange::default());
}

#[test]
fn hot_reload_state_migration_rejects_duplicate_stable_keys() {
    let tree_id = UiTreeId::new("runtime.ui.duplicate-state-key");
    let mut previous_tree = UiTree::new(tree_id.clone());
    previous_tree.insert_root(state_node(1, "Shared", "TextInput"));
    previous_tree.insert_root(state_node(2, "Shared", "TextInput"));
    let mut replacement_tree = UiTree::new(tree_id);
    replacement_tree.insert_root(state_node(10, "Shared", "TextInput"));

    let mut previous = UiSurfaceComponentStateStore::default();
    previous.set_value(
        UiNodeId::new(1),
        "text",
        UiValue::String("first".to_string()),
    );
    previous.set_value(
        UiNodeId::new(2),
        "text",
        UiValue::String("second".to_string()),
    );
    let mut replacement = UiSurfaceComponentStateStore::default();

    let report = replacement.migrate_stable_from(&previous, &previous_tree, &replacement_tree);

    assert_eq!(report.migrated, 0);
    assert_eq!(report.reset, 2);
    assert!(replacement.get(UiNodeId::new(10)).is_none());
}

fn state_node(node_id: u64, control_id: &str, component: &str) -> UiTreeNode {
    UiTreeNode::new(
        UiNodeId::new(node_id),
        UiNodePath::new(format!("reload/{node_id}")),
    )
    .with_template_metadata(UiTemplateNodeMetadata {
        component: component.to_string(),
        control_id: Some(control_id.to_string()),
        ..Default::default()
    })
}
