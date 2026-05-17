use accesskit::{Action, ActionData, ActionRequest, Node, NodeId, Rect, Role, Toggled};
use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yCheckedState, UiA11yRole, UiA11yState, UiAccessibilityAction,
        UiAccessibilityActionSource, UiAccessibilityNode, UiAccessibilityTreeSnapshot,
    },
    event_ui::{UiNodeId, UiTreeId},
    layout::UiFrame,
};

use crate::ui::accessibility::accesskit::{
    accesskit_role, neutral_action_request_from_accesskit, snapshot_to_accesskit_tree_update,
};

fn id(value: u64) -> UiNodeId {
    UiNodeId::new(value)
}

fn node(update_node: &(NodeId, Node)) -> &Node {
    &update_node.1
}

fn find_node(update: &accesskit::TreeUpdate, id: u64) -> &Node {
    node(
        update
            .nodes
            .iter()
            .find(|(node_id, _)| *node_id == NodeId(id))
            .expect("accesskit node"),
    )
}

#[test]
fn accesskit_tree_update_maps_roles_actions_bounds_children_and_focus() {
    let snapshot = UiAccessibilityTreeSnapshot {
        tree_id: UiTreeId::new("runtime.ui.accesskit"),
        roots: vec![id(1)],
        nodes: vec![
            UiAccessibilityNode {
                node_id: id(1),
                role: UiA11yRole::Panel,
                name: Some("Root".to_string()),
                children: vec![id(2), id(3)],
                bounds: Some(UiFrame::new(0.0, 0.0, 200.0, 100.0)),
                ..UiAccessibilityNode::default()
            },
            UiAccessibilityNode {
                node_id: id(2),
                role: UiA11yRole::Button,
                name: Some("Save".to_string()),
                actions: vec![
                    UiAccessibilityAction::Activate,
                    UiAccessibilityAction::Focus,
                ],
                bounds: Some(UiFrame::new(8.0, 12.0, 80.0, 24.0)),
                ..UiAccessibilityNode::default()
            },
            UiAccessibilityNode {
                node_id: id(3),
                role: UiA11yRole::Checkbox,
                name: Some("Enabled".to_string()),
                state: UiA11yState {
                    checked: Some(UiA11yCheckedState::True),
                    selected: true,
                    ..UiA11yState::default()
                },
                actions: vec![UiAccessibilityAction::Activate],
                bounds: Some(UiFrame::new(8.0, 44.0, 80.0, 24.0)),
                ..UiAccessibilityNode::default()
            },
        ],
        focused: Some(id(2)),
        diagnostics: Vec::new(),
    };

    let update = snapshot_to_accesskit_tree_update(&snapshot).expect("tree update");

    assert_eq!(update.tree.as_ref().unwrap().root, NodeId(1));
    assert_eq!(update.focus, NodeId(2));
    assert_eq!(update.nodes.len(), 3);

    let root = find_node(&update, 1);
    assert_eq!(root.role(), Role::Pane);
    assert_eq!(root.label(), Some("Root"));
    assert_eq!(root.children(), &[NodeId(2), NodeId(3)]);

    let button = find_node(&update, 2);
    assert_eq!(button.role(), Role::Button);
    assert_eq!(button.label(), Some("Save"));
    assert!(button.supports_action(Action::Click));
    assert!(button.supports_action(Action::Focus));
    assert_eq!(
        button.bounds(),
        Some(Rect {
            x0: 8.0,
            y0: 12.0,
            x1: 88.0,
            y1: 36.0,
        })
    );

    let checkbox = find_node(&update, 3);
    assert_eq!(checkbox.role(), Role::CheckBox);
    assert_eq!(checkbox.toggled(), Some(Toggled::True));
    assert_eq!(checkbox.is_selected(), Some(true));
}

#[test]
fn accesskit_tree_update_preserves_text_values_slider_numeric_state_and_relations() {
    let snapshot = UiAccessibilityTreeSnapshot {
        tree_id: UiTreeId::new("runtime.ui.accesskit.relations"),
        roots: vec![id(1)],
        nodes: vec![
            UiAccessibilityNode {
                node_id: id(1),
                role: UiA11yRole::Panel,
                children: vec![id(2), id(3), id(4)],
                ..UiAccessibilityNode::default()
            },
            UiAccessibilityNode {
                node_id: id(2),
                role: UiA11yRole::Text,
                name: Some("Volume".to_string()),
                label_for: Some(id(3)),
                ..UiAccessibilityNode::default()
            },
            UiAccessibilityNode {
                node_id: id(3),
                role: UiA11yRole::Slider,
                name: Some("Volume slider".to_string()),
                labelled_by: Some(id(2)),
                tooltip: Some("Adjust volume".to_string()),
                state: UiA11yState {
                    value: Some("0.5".to_string()),
                    expanded: Some(true),
                    ..UiA11yState::default()
                },
                actions: vec![
                    UiAccessibilityAction::Focus,
                    UiAccessibilityAction::Increment,
                    UiAccessibilityAction::Decrement,
                    UiAccessibilityAction::SetValue,
                ],
                ..UiAccessibilityNode::default()
            },
            UiAccessibilityNode {
                node_id: id(4),
                role: UiA11yRole::TextInput,
                name: Some("Search".to_string()),
                state: UiA11yState {
                    value: Some("query".to_string()),
                    disabled: true,
                    ..UiA11yState::default()
                },
                actions: vec![UiAccessibilityAction::Focus],
                ..UiAccessibilityNode::default()
            },
        ],
        focused: Some(id(99)),
        diagnostics: Vec::new(),
    };

    let update = snapshot_to_accesskit_tree_update(&snapshot).expect("tree update");

    assert_eq!(update.focus, NodeId(1));

    let label = find_node(&update, 2);
    assert_eq!(label.role(), Role::Label);
    assert_eq!(label.value(), Some("Volume"));
    assert_eq!(label.controls(), &[NodeId(3)]);

    let slider = find_node(&update, 3);
    assert_eq!(slider.role(), Role::Slider);
    assert_eq!(slider.labelled_by(), &[NodeId(2)]);
    assert_eq!(slider.tooltip(), Some("Adjust volume"));
    assert_eq!(slider.value(), Some("0.5"));
    assert_eq!(slider.numeric_value(), Some(0.5));
    assert_eq!(slider.is_expanded(), Some(true));
    assert!(slider.supports_action(Action::Increment));
    assert!(slider.supports_action(Action::Decrement));
    assert!(slider.supports_action(Action::SetValue));

    let input = find_node(&update, 4);
    assert_eq!(input.role(), Role::TextInput);
    assert_eq!(input.label(), Some("Search"));
    assert_eq!(input.value(), Some("query"));
    assert!(input.is_disabled());
    assert!(input.supports_action(Action::Focus));
}

#[test]
fn accesskit_bridge_maps_action_requests_back_to_neutral_accessibility_actions() {
    let set_value = neutral_action_request_from_accesskit(&ActionRequest {
        target: NodeId(42),
        action: Action::SetValue,
        data: Some(ActionData::Value("42".into())),
    })
    .expect("set value request");
    assert_eq!(set_value.target, id(42));
    assert_eq!(set_value.action, UiAccessibilityAction::SetValue);
    assert_eq!(
        set_value.source,
        UiAccessibilityActionSource::AssistiveTechnology
    );
    assert_eq!(set_value.value.as_deref(), Some("42"));
    assert_eq!(set_value.numeric_value, None);

    let numeric = neutral_action_request_from_accesskit(&ActionRequest {
        target: NodeId(7),
        action: Action::SetValue,
        data: Some(ActionData::NumericValue(0.75)),
    })
    .expect("numeric set value request");
    assert_eq!(numeric.target, id(7));
    assert_eq!(numeric.action, UiAccessibilityAction::SetValue);
    assert_eq!(numeric.numeric_value, Some(0.75));

    let increment = neutral_action_request_from_accesskit(&ActionRequest {
        target: NodeId(7),
        action: Action::Increment,
        data: None,
    })
    .expect("increment request");
    assert_eq!(increment.action, UiAccessibilityAction::Increment);

    let dismiss = neutral_action_request_from_accesskit(&ActionRequest {
        target: NodeId(8),
        action: Action::HideTooltip,
        data: None,
    })
    .expect("dismiss request");
    assert_eq!(dismiss.action, UiAccessibilityAction::Dismiss);

    assert!(neutral_action_request_from_accesskit(&ActionRequest {
        target: NodeId(9),
        action: Action::ShowContextMenu,
        data: None,
    })
    .is_none());
}

#[test]
fn accesskit_role_mapping_covers_zircon_accessibility_roles() {
    assert_eq!(accesskit_role(UiA11yRole::Button), Role::Button);
    assert_eq!(accesskit_role(UiA11yRole::Checkbox), Role::CheckBox);
    assert_eq!(accesskit_role(UiA11yRole::Radio), Role::RadioButton);
    assert_eq!(accesskit_role(UiA11yRole::RadioGroup), Role::RadioGroup);
    assert_eq!(accesskit_role(UiA11yRole::Slider), Role::Slider);
    assert_eq!(accesskit_role(UiA11yRole::Text), Role::Label);
    assert_eq!(accesskit_role(UiA11yRole::TextInput), Role::TextInput);
    assert_eq!(accesskit_role(UiA11yRole::Image), Role::Image);
    assert_eq!(accesskit_role(UiA11yRole::Panel), Role::Pane);
    assert_eq!(accesskit_role(UiA11yRole::Dialog), Role::Dialog);
    assert_eq!(accesskit_role(UiA11yRole::Tooltip), Role::Tooltip);
    assert_eq!(accesskit_role(UiA11yRole::Scrollbar), Role::ScrollBar);
}
