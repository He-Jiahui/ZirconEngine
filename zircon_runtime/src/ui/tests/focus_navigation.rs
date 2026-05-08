use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
    tree::UiRuntimeTreeAccessExt,
};
use zircon_runtime_interface::ui::{
    component::UiValue,
    dispatch::{
        UiInputEvent, UiInputEventMetadata, UiInputSequence, UiInputTimestamp,
        UiKeyboardInputEvent, UiKeyboardInputState, UiPointerId,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    focus::{UiFocusChangeReason, UiFocusVisibleReason, UiFocusedInputKind},
    layout::UiFrame,
    navigation::{
        UiDirectionalNavigation, UiDirectionalNavigationTarget, UiNavigationContract,
        UiNavigationGroup, UiNavigationGroupId, UiTabIndex,
    },
    surface::UiNavigationEventKind,
    tree::{UiInputPolicy, UiTreeNode},
};

#[test]
fn autofocus_records_initial_focus_change_and_visible_reason() {
    let mut surface = focus_surface();

    let event = surface.resolve_autofocus().unwrap().expect("autofocus");

    assert_eq!(surface.focus.focused, Some(id(2)));
    assert_eq!(surface.focus.pending_autofocus, None);
    assert_eq!(surface.focus.previous, None);
    assert_eq!(event.current, Some(id(2)));
    assert_eq!(event.reason, UiFocusChangeReason::Autofocus);
    assert!(event.visible.visible);
    assert_eq!(event.visible.reason, UiFocusVisibleReason::Programmatic);
    assert_eq!(surface.focus.changes, vec![event]);
}

#[test]
fn pointer_and_navigation_focus_sources_update_visible_reason() {
    let mut surface = focus_surface();

    surface.focus_node(id(2)).unwrap();
    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Next,
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(3)));
    assert!(surface.focus.focus_visible.visible);
    assert_eq!(
        surface.focus.focus_visible.reason,
        UiFocusVisibleReason::KeyboardNavigation
    );

    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Keyboard(UiKeyboardInputEvent {
                metadata: input_metadata(),
                state: UiKeyboardInputState::Pressed,
                key_code: 65,
                scan_code: Some(30),
                physical_key: "KeyA".to_string(),
                logical_key: "A".to_string(),
                text: Some("a".to_string()),
            }),
        )
        .unwrap();

    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(surface.focus.focused_inputs[0].focused, id(3));
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Keyboard
    );
    assert_eq!(surface.focus.focused_inputs[0].route, vec![id(3), id(1)]);
    assert!(surface.focus.focused_inputs[0].accepted);
}

#[test]
fn text_and_ime_inputs_record_focused_input_routes() {
    let mut surface = focus_surface();
    surface.focus_node(id(2)).unwrap();
    surface.input.input_method_owner = Some(id(2));

    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Text(zircon_runtime_interface::ui::dispatch::UiTextInputEvent {
                metadata: input_metadata(),
                text: "x".to_string(),
            }),
        )
        .unwrap();
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Ime(zircon_runtime_interface::ui::dispatch::UiImeInputEvent {
                metadata: input_metadata(),
                kind: zircon_runtime_interface::ui::dispatch::UiImeInputEventKind::Cancel,
                text: String::new(),
                cursor_range: None,
            }),
        )
        .unwrap();

    assert_eq!(surface.focus.focused_inputs.len(), 2);
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Text
    );
    assert_eq!(
        surface.focus.focused_inputs[1].kind,
        UiFocusedInputKind::Ime
    );
    assert_eq!(surface.focus.focused_inputs[0].route, vec![id(2), id(1)]);
    assert_eq!(surface.focus.focused_inputs[1].route, vec![id(2), id(1)]);
    assert!(surface.focus.focused_inputs[0].accepted);
    assert!(surface.focus.focused_inputs[1].accepted);
}

#[test]
fn authored_focus_contract_makes_node_focusable_without_legacy_state_flag() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.focus.contract"));
    surface.tree.insert_root(root_node());
    surface
        .tree
        .insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/authored_focus"))
                .with_frame(UiFrame::new(0.0, 0.0, 32.0, 24.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: false,
                    hoverable: false,
                    focusable: false,
                    ..Default::default()
                })
                .with_focus_contract({
                    let mut focus = zircon_runtime_interface::ui::focus::UiFocusContract::default();
                    focus.focusable = true;
                    focus
                }),
        )
        .unwrap();
    surface.rebuild();

    surface.focus_node(id(2)).unwrap();
    let arranged = surface.arranged_tree.get(id(2)).unwrap();

    assert_eq!(surface.focus.focused, Some(id(2)));
    assert!(arranged.focusable);
    assert!(arranged.supports_pointer());
}

#[test]
fn focus_is_cleared_when_focused_node_stops_accepting_input() {
    let mut surface = focus_surface();
    surface.focus_node(id(2)).unwrap();
    surface.input.input_method_owner = Some(id(2));
    surface.focus.captured = Some(id(2));
    surface.input.captured_pointer_id = Some(UiPointerId::new(7));

    let event = surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(2),
            "enabled",
            zircon_runtime_interface::ui::component::UiValue::Bool(false),
        ))
        .unwrap()
        .focus_change
        .expect("focus cleared");

    assert_eq!(surface.focus.focused, None);
    assert_eq!(surface.input.input_method_owner, None);
    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.input.captured_pointer_id, None);
    assert_eq!(event.previous, Some(id(2)));
    assert_eq!(event.current, None);
    assert_eq!(event.reason, UiFocusChangeReason::Disabled);
}

#[test]
fn focus_is_cleared_when_focused_node_ancestor_is_disabled() {
    let mut surface = focus_surface();
    surface.focus_node(id(2)).unwrap();
    surface.input.input_method_owner = Some(id(2));
    surface.focus.captured = Some(id(2));
    surface.input.pointer_lock_owner = Some(id(2));

    let event = surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(1),
            "enabled",
            UiValue::Bool(false),
        ))
        .unwrap()
        .focus_change
        .expect("ancestor disabled clears descendant focus");

    assert_eq!(surface.focus.focused, None);
    assert_eq!(surface.input.input_method_owner, None);
    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.input.pointer_lock_owner, None);
    assert_eq!(event.previous, Some(id(2)));
    assert_eq!(event.current, None);
    assert_eq!(event.reason, UiFocusChangeReason::Disabled);
}

#[test]
fn unchanged_or_rejected_focus_related_mutations_do_not_emit_focus_changes() {
    let mut surface = focus_surface();
    surface.focus_node(id(2)).unwrap();

    let unchanged = surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(2),
            "enabled",
            UiValue::Bool(true),
        ))
        .unwrap();
    let rejected = surface
        .mutate_property(crate::ui::surface::UiPropertyMutationRequest::new(
            id(2),
            "focusable",
            UiValue::String("false".to_string()),
        ))
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(2)));
    assert!(unchanged.focus_change.is_none());
    assert!(rejected.focus_change.is_none());
}

#[test]
fn tab_navigation_uses_index_order_and_modal_group_trap() {
    let mut surface = navigation_surface();
    surface.focus_node(id(2)).unwrap();

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Next,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(3)));

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Next,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(2)));

    surface.focus_node(id(5)).unwrap();
    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Next,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(6)));

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Previous,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(5)));
}

#[test]
fn tab_navigation_crosses_non_modal_groups_by_group_order() {
    let mut surface = non_modal_group_surface();
    surface.focus_node(id(2)).unwrap();

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Next,
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(5)));
}

#[test]
fn directional_navigation_honors_manual_overrides_and_blocked_edges() {
    let mut surface = navigation_surface();
    surface.focus_node(id(2)).unwrap();

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Right,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(5)));

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Left,
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(id(5)));
}

#[test]
fn modal_directional_navigation_rejects_manual_targets_outside_modal_group() {
    let mut surface = navigation_surface();
    let modal_b = surface.tree.nodes.get_mut(&id(6)).unwrap();
    modal_b.navigation.directional = Some(UiDirectionalNavigation {
        right: UiDirectionalNavigationTarget::Node(id(2)),
        ..Default::default()
    });
    surface.focus_node(id(6)).unwrap();

    surface
        .dispatch_navigation_event(
            &UiNavigationDispatcher::default(),
            UiNavigationEventKind::Right,
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(id(6)));
}

fn focus_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.focus.m2"));
    surface.tree.insert_root(root_node());
    surface
        .tree
        .insert_child(
            id(1),
            focus_node(2, "first", 0.0, 0.0).with_focus_contract({
                let mut focus = zircon_runtime_interface::ui::focus::UiFocusContract::default();
                focus.focusable = true;
                focus.autofocus = true;
                focus
            }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(id(1), focus_node(3, "second", 90.0, 0.0))
        .unwrap();
    surface.rebuild();
    surface
}

fn navigation_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.navigation.m3"));
    surface.tree.insert_root(root_node());
    surface
        .tree
        .insert_child(
            id(1),
            focus_node(2, "two", 0.0, 0.0).with_navigation_contract({
                let mut navigation = navigation_contract(2, 20);
                navigation.directional = Some(UiDirectionalNavigation {
                    right: UiDirectionalNavigationTarget::Node(id(5)),
                    ..Default::default()
                });
                navigation
            }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            focus_node(3, "three", 40.0, 0.0).with_navigation_contract(navigation_contract(1, 10)),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            focus_node(5, "modal_a", 0.0, 50.0).with_navigation_contract({
                let mut navigation = navigation_contract(1, 0);
                navigation.group = Some(UiNavigationGroup {
                    group_id: UiNavigationGroupId::new("dialog"),
                    root: Some(id(5)),
                    modal: true,
                    wrap: true,
                    order: 0,
                    ..Default::default()
                });
                navigation.directional = Some(UiDirectionalNavigation {
                    left: UiDirectionalNavigationTarget::Blocked,
                    right: UiDirectionalNavigationTarget::Node(id(6)),
                    ..Default::default()
                });
                navigation
            }),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            focus_node(6, "modal_b", 40.0, 50.0).with_navigation_contract({
                let mut navigation = navigation_contract(2, 0);
                navigation.group = Some(UiNavigationGroup {
                    group_id: UiNavigationGroupId::new("dialog"),
                    parent: None,
                    root: Some(id(5)),
                    modal: true,
                    wrap: true,
                    order: 0,
                });
                navigation
            }),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn non_modal_group_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.navigation.groups"));
    surface.tree.insert_root(root_node());
    surface
        .tree
        .insert_child(
            id(1),
            focus_node(2, "root_a", 0.0, 0.0).with_navigation_contract(navigation_contract(2, 0)),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            focus_node(3, "root_b", 40.0, 0.0).with_navigation_contract(navigation_contract(1, 0)),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            id(1),
            focus_node(5, "tools", 80.0, 0.0).with_navigation_contract({
                let mut navigation = navigation_contract(1, 10);
                navigation.group = Some(UiNavigationGroup {
                    group_id: UiNavigationGroupId::new("tools"),
                    root: Some(id(1)),
                    modal: false,
                    wrap: true,
                    order: 10,
                    ..Default::default()
                });
                navigation
            }),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn root_node() -> UiTreeNode {
    UiTreeNode::new(id(1), UiNodePath::new("root")).with_frame(UiFrame::new(0.0, 0.0, 180.0, 120.0))
}

fn focus_node(id_value: u64, path: &str, x: f32, y: f32) -> UiTreeNode {
    let metadata = zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata {
        component: "TextField".to_string(),
        control_id: Some(path.to_string()),
        attributes: [
            ("editable_text".to_string(), toml::Value::Boolean(true)),
            ("value".to_string(), toml::Value::String(String::new())),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    };
    UiTreeNode::new(id(id_value), UiNodePath::new(format!("root/{path}")))
        .with_frame(UiFrame::new(x, y, 32.0, 24.0))
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(UiStateFlags {
            visible: true,
            enabled: true,
            clickable: true,
            hoverable: true,
            focusable: true,
            ..Default::default()
        })
        .with_template_metadata(metadata)
}

fn navigation_contract(order: i32, group_order: i32) -> UiNavigationContract {
    UiNavigationContract {
        tab_index: Some(UiTabIndex::new(order)),
        group: Some(UiNavigationGroup {
            group_id: UiNavigationGroupId::new("root"),
            root: Some(id(1)),
            modal: false,
            wrap: true,
            order: group_order,
            ..Default::default()
        }),
        directional: None,
    }
}

fn input_metadata() -> UiInputEventMetadata {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(10), UiInputSequence::new(1));
    metadata.pointer_id = Some(UiPointerId::new(7));
    metadata
}

fn id(value: u64) -> UiNodeId {
    UiNodeId::new(value)
}
