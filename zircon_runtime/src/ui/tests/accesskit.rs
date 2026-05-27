use accesskit::{
    Action, ActionData, ActionRequest, Node, NodeId, Point, Rect, Role, TextPosition,
    TextSelection, Toggled,
};
use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yCheckedState, UiA11yRole, UiA11yState, UiA11yTextSelection, UiAccessibilityAction,
        UiAccessibilityActionSource, UiAccessibilityNode, UiAccessibilityTreeSnapshot,
    },
    event_ui::{UiNodeId, UiTreeId},
    layout::{UiFrame, UiPoint},
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
                    UiAccessibilityAction::Expand,
                ],
                state: UiA11yState {
                    expanded: Some(false),
                    ..UiA11yState::default()
                },
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
    assert!(button.supports_action(Action::Expand));
    assert_eq!(button.is_expanded(), Some(false));
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
                children: vec![id(2), id(3), id(4), id(5)],
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
            UiAccessibilityNode {
                node_id: id(5),
                role: UiA11yRole::Panel,
                name: Some("Scrollable results".to_string()),
                actions: vec![UiAccessibilityAction::ScrollTo],
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

    let scrollable = find_node(&update, 5);
    assert!(scrollable.supports_action(Action::ScrollIntoView));
    assert!(scrollable.supports_action(Action::SetScrollOffset));
}

#[test]
fn accesskit_tree_update_maps_dismiss_actions_by_role() {
    let snapshot = UiAccessibilityTreeSnapshot {
        tree_id: UiTreeId::new("runtime.ui.accesskit.dismiss"),
        roots: vec![id(1)],
        nodes: vec![
            UiAccessibilityNode {
                node_id: id(1),
                role: UiA11yRole::Panel,
                children: vec![id(2), id(3)],
                ..UiAccessibilityNode::default()
            },
            UiAccessibilityNode {
                node_id: id(2),
                role: UiA11yRole::Dialog,
                name: Some("Inspector".to_string()),
                actions: vec![UiAccessibilityAction::Dismiss],
                ..UiAccessibilityNode::default()
            },
            UiAccessibilityNode {
                node_id: id(3),
                role: UiA11yRole::Tooltip,
                name: Some("Hint".to_string()),
                actions: vec![UiAccessibilityAction::Dismiss],
                ..UiAccessibilityNode::default()
            },
        ],
        ..UiAccessibilityTreeSnapshot::default()
    };

    let update = snapshot_to_accesskit_tree_update(&snapshot).expect("tree update");

    let dialog = find_node(&update, 2);
    assert_eq!(dialog.role(), Role::Dialog);
    assert!(dialog.supports_action(Action::Blur));
    assert!(!dialog.supports_action(Action::HideTooltip));

    let tooltip = find_node(&update, 3);
    assert_eq!(tooltip.role(), Role::Tooltip);
    assert!(tooltip.supports_action(Action::HideTooltip));
    assert!(!tooltip.supports_action(Action::Blur));
}

#[test]
fn accesskit_bridge_maps_action_requests_back_to_neutral_accessibility_actions() {
    let empty_snapshot = UiAccessibilityTreeSnapshot::default();

    let set_value = neutral_action_request_from_accesskit(
        &ActionRequest {
            target: NodeId(42),
            action: Action::SetValue,
            data: Some(ActionData::Value("42".into())),
        },
        &empty_snapshot,
    )
    .expect("set value request");
    assert_eq!(set_value.target, id(42));
    assert_eq!(set_value.action, UiAccessibilityAction::SetValue);
    assert_eq!(
        set_value.source,
        UiAccessibilityActionSource::AssistiveTechnology
    );
    assert_eq!(set_value.value.as_deref(), Some("42"));
    assert_eq!(set_value.numeric_value, None);

    let numeric = neutral_action_request_from_accesskit(
        &ActionRequest {
            target: NodeId(7),
            action: Action::SetValue,
            data: Some(ActionData::NumericValue(0.75)),
        },
        &empty_snapshot,
    )
    .expect("numeric set value request");
    assert_eq!(numeric.target, id(7));
    assert_eq!(numeric.action, UiAccessibilityAction::SetValue);
    assert_eq!(numeric.numeric_value, Some(0.75));

    let replace_selected_text = neutral_action_request_from_accesskit(
        &ActionRequest {
            target: NodeId(43),
            action: Action::ReplaceSelectedText,
            data: Some(ActionData::Value("replacement".into())),
        },
        &empty_snapshot,
    )
    .expect("replace selected text request");
    assert_eq!(replace_selected_text.target, id(43));
    assert_eq!(
        replace_selected_text.action,
        UiAccessibilityAction::ReplaceSelectedText
    );
    assert_eq!(replace_selected_text.value.as_deref(), Some("replacement"));

    let text_selection_snapshot = UiAccessibilityTreeSnapshot {
        nodes: vec![UiAccessibilityNode {
            node_id: id(44),
            role: UiA11yRole::TextInput,
            state: UiA11yState {
                value: Some("a\u{00e9}b".to_string()),
                ..UiA11yState::default()
            },
            ..UiAccessibilityNode::default()
        }],
        ..UiAccessibilityTreeSnapshot::default()
    };
    let set_text_selection = neutral_action_request_from_accesskit(
        &ActionRequest {
            target: NodeId(44),
            action: Action::SetTextSelection,
            data: Some(ActionData::SetTextSelection(TextSelection {
                anchor: TextPosition {
                    node: NodeId(44),
                    character_index: 1,
                },
                focus: TextPosition {
                    node: NodeId(44),
                    character_index: 2,
                },
            })),
        },
        &text_selection_snapshot,
    )
    .expect("set text selection request");
    assert_eq!(set_text_selection.target, id(44));
    assert_eq!(
        set_text_selection.action,
        UiAccessibilityAction::SetTextSelection
    );
    assert_eq!(
        set_text_selection.text_selection,
        Some(UiA11yTextSelection {
            caret: 3,
            anchor: 1,
            focus: 3,
        })
    );
    assert!(neutral_action_request_from_accesskit(
        &ActionRequest {
            target: NodeId(44),
            action: Action::SetTextSelection,
            data: Some(ActionData::SetTextSelection(TextSelection {
                anchor: TextPosition {
                    node: NodeId(44),
                    character_index: 1,
                },
                focus: TextPosition {
                    node: NodeId(44),
                    character_index: 2,
                },
            })),
        },
        &empty_snapshot
    )
    .is_none());

    let set_scroll_offset = neutral_action_request_from_accesskit(
        &ActionRequest {
            target: NodeId(45),
            action: Action::SetScrollOffset,
            data: Some(ActionData::SetScrollOffset(Point { x: 24.0, y: 64.0 })),
        },
        &empty_snapshot,
    )
    .expect("set scroll offset request");
    assert_eq!(set_scroll_offset.target, id(45));
    assert_eq!(set_scroll_offset.action, UiAccessibilityAction::ScrollTo);
    assert_eq!(
        set_scroll_offset.scroll_offset,
        Some(UiPoint::new(24.0, 64.0))
    );

    let increment = neutral_action_request_from_accesskit(
        &ActionRequest {
            target: NodeId(7),
            action: Action::Increment,
            data: None,
        },
        &empty_snapshot,
    )
    .expect("increment request");
    assert_eq!(increment.action, UiAccessibilityAction::Increment);

    let expand = neutral_action_request_from_accesskit(
        &ActionRequest {
            target: NodeId(46),
            action: Action::Expand,
            data: None,
        },
        &empty_snapshot,
    )
    .expect("expand request");
    assert_eq!(expand.target, id(46));
    assert_eq!(expand.action, UiAccessibilityAction::Expand);

    let collapse = neutral_action_request_from_accesskit(
        &ActionRequest {
            target: NodeId(47),
            action: Action::Collapse,
            data: None,
        },
        &empty_snapshot,
    )
    .expect("collapse request");
    assert_eq!(collapse.target, id(47));
    assert_eq!(collapse.action, UiAccessibilityAction::Collapse);

    let dismiss = neutral_action_request_from_accesskit(
        &ActionRequest {
            target: NodeId(8),
            action: Action::HideTooltip,
            data: None,
        },
        &empty_snapshot,
    )
    .expect("dismiss request");
    assert_eq!(dismiss.action, UiAccessibilityAction::Dismiss);

    let blur_dismiss = neutral_action_request_from_accesskit(
        &ActionRequest {
            target: NodeId(11),
            action: Action::Blur,
            data: None,
        },
        &empty_snapshot,
    )
    .expect("blur dismiss request");
    assert_eq!(blur_dismiss.target, id(11));
    assert_eq!(blur_dismiss.action, UiAccessibilityAction::Dismiss);

    assert!(neutral_action_request_from_accesskit(
        &ActionRequest {
            target: NodeId(9),
            action: Action::ShowContextMenu,
            data: None,
        },
        &empty_snapshot
    )
    .is_none());
    assert!(neutral_action_request_from_accesskit(
        &ActionRequest {
            target: NodeId(10),
            action: Action::ScrollToPoint,
            data: Some(ActionData::ScrollToPoint(Point { x: 10.0, y: 20.0 })),
        },
        &empty_snapshot
    )
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
