use crate::ui::surface::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface};
use zircon_runtime_interface::ui::{
    component::UiValue,
    dispatch::{
        UiDispatchEffect, UiDispatchReply, UiInputEvent, UiInputEventMetadata, UiInputSequence,
        UiInputTimestamp, UiKeyboardInputEvent, UiKeyboardInputState, UiPopupEffectKind,
        UiTransientDismissalReason, UiTransientDismissalTarget,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
    widget::{UiPopupAnchor, UiWidgetBehavior, UiWidgetContract},
};

#[test]
fn declarative_popup_parent_close_clears_nested_state_before_rebuild() {
    let mut surface = declarative_popup_surface(true);
    insert_nested_popup(&mut surface);
    for node_id in [node(2), node(4)] {
        surface
            .tree
            .node_mut(node_id)
            .unwrap()
            .template_metadata
            .as_mut()
            .unwrap()
            .attributes
            .insert("open".to_string(), toml::Value::Boolean(true));
    }

    surface
        .mutate_property(UiPropertyMutationRequest::new(
            node(2),
            "popup_open",
            UiValue::Bool(false),
        ))
        .unwrap();

    for node_id in [node(2), node(4)] {
        assert_popup_open(&surface, node_id, false);
        assert_popup_open_alias(&surface, node_id, false);
    }
    assert_popup_stack(&surface, &[]);
    surface.rebuild();
    assert_popup_stack(&surface, &[]);
}

#[test]
fn declarative_popup_parent_close_clears_stacked_sibling_before_rebuild() {
    let mut surface = declarative_popup_surface(true);
    insert_popup(&mut surface, node(1), node(4), "root/sibling-popup", true);
    surface.rebuild();
    assert_popup_stack(&surface, &["root/popup", "root/sibling-popup"]);

    surface
        .mutate_property(UiPropertyMutationRequest::new(
            node(2),
            "popup_open",
            UiValue::Bool(false),
        ))
        .unwrap();

    assert_popup_open(&surface, node(2), false);
    assert_popup_open(&surface, node(4), false);
    assert_popup_stack(&surface, &[]);
    surface.rebuild();
    assert_popup_stack(&surface, &[]);
}

#[test]
fn declarative_popup_parent_close_clears_unstacked_descendants_before_rebuild() {
    let mut surface = declarative_popup_surface(true);
    insert_deep_nested_popup(&mut surface);
    surface.input.popup_stack.clear();

    surface
        .mutate_property(UiPropertyMutationRequest::new(
            node(2),
            "popup_open",
            UiValue::Bool(false),
        ))
        .unwrap();

    for node_id in [node(2), node(4), node(5)] {
        assert_popup_open(&surface, node_id, false);
    }
    surface.rebuild();
    assert_popup_stack(&surface, &[]);
}

#[test]
fn declarative_popup_branch_close_orders_unstacked_deep_child_first() {
    let mut surface = declarative_popup_surface(true);
    insert_deep_nested_popup(&mut surface);
    surface
        .input
        .popup_stack
        .retain(|popup| popup.popup_node != Some(node(5)));

    assert_eq!(
        surface
            .popup_branch_closures(node(2))
            .into_iter()
            .map(|(node_id, _)| node_id)
            .collect::<Vec<_>>(),
        vec![node(5), node(4)]
    );
}

#[test]
fn declarative_popup_transient_dismissal_clears_state_before_rebuild() {
    let mut surface = declarative_popup_surface(true);
    insert_deep_nested_popup(&mut surface);

    let result = surface.apply_dispatch_reply(
        keyboard_event("Escape", 27),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::DismissTransientUi {
            target: UiTransientDismissalTarget::PopupStack,
            reason: UiTransientDismissalReason::Programmatic,
        }),
    );

    assert!(result.rejected_effects.is_empty());
    for node_id in [node(2), node(4), node(5)] {
        assert_popup_open(&surface, node_id, false);
    }
    surface.rebuild();
    assert_popup_stack(&surface, &[]);
}

#[test]
fn declarative_popup_close_effect_clears_branch_when_stack_is_present_or_missing() {
    for stack_present in [true, false] {
        let mut surface = declarative_popup_surface(true);
        insert_deep_nested_popup(&mut surface);
        if !stack_present {
            surface.input.popup_stack.clear();
        }

        let result = surface.apply_dispatch_reply(
            keyboard_event("Escape", 27),
            popup_effect(UiPopupEffectKind::Close),
        );

        assert!(result.rejected_effects.is_empty());
        for node_id in [node(2), node(4), node(5)] {
            assert_popup_open(&surface, node_id, false);
        }
        surface.rebuild();
        assert_popup_stack(&surface, &[]);
    }
}

#[test]
fn declarative_popup_open_and_toggle_use_popup_open_without_creating_open_alias() {
    let mut surface = declarative_popup_surface(false);

    let opened = surface.apply_dispatch_reply(
        keyboard_event("Enter", 13),
        popup_effect(UiPopupEffectKind::Open),
    );
    assert!(opened.rejected_effects.is_empty());
    assert_popup_open(&surface, node(2), true);
    assert!(surface
        .tree
        .node(node(2))
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes
        .get("open")
        .is_none());
    assert_popup_stack(&surface, &["root/popup"]);

    let toggled = surface.apply_dispatch_reply(
        keyboard_event("Enter", 13),
        popup_effect(UiPopupEffectKind::Toggle),
    );
    assert!(toggled.rejected_effects.is_empty());
    assert_popup_open(&surface, node(2), false);
    assert_popup_stack(&surface, &[]);
}

#[test]
fn declarative_popup_open_restores_missing_stack_and_preserves_nested_entries() {
    let mut surface = declarative_popup_surface(true);
    insert_nested_popup(&mut surface);
    surface.input.popup_stack.clear();

    let result = surface.apply_dispatch_reply(
        keyboard_event("Enter", 13),
        popup_effect(UiPopupEffectKind::Open),
    );

    assert!(result.rejected_effects.is_empty());
    assert_popup_stack(&surface, &["root/popup"]);

    surface.rebuild();
    assert_popup_stack(&surface, &["root/popup", "root/popup/nested"]);

    let repeat_open = surface.apply_dispatch_reply(
        keyboard_event("Enter", 13),
        popup_effect(UiPopupEffectKind::Open),
    );
    assert!(repeat_open.rejected_effects.is_empty());
    assert_popup_stack(&surface, &["root/popup", "root/popup/nested"]);
}

#[test]
fn declarative_popup_open_restores_missing_parent_before_existing_nested_entry() {
    let mut surface = declarative_popup_surface(true);
    insert_nested_popup(&mut surface);
    surface.input.popup_stack.remove(0);
    assert_popup_stack(&surface, &["root/popup/nested"]);

    let result = surface.apply_dispatch_reply(
        keyboard_event("Enter", 13),
        popup_effect(UiPopupEffectKind::Open),
    );

    assert!(result.rejected_effects.is_empty());
    assert_popup_stack(&surface, &["root/popup", "root/popup/nested"]);
    surface.rebuild();
    assert_popup_stack(&surface, &["root/popup", "root/popup/nested"]);
}

#[test]
fn declarative_popup_close_uses_node_when_runtime_id_is_stale() {
    let mut surface = declarative_popup_surface(true);
    surface.input.popup_stack[0].popup_id = "legacy/popup".to_string();

    let result = surface.apply_dispatch_reply(
        keyboard_event("Escape", 27),
        popup_effect(UiPopupEffectKind::Close),
    );

    assert!(result.rejected_effects.is_empty());
    assert_eq!(result.diagnostics.route_target, Some(node(2)));
    assert_popup_open(&surface, node(2), false);
    assert_popup_stack(&surface, &[]);
}

#[test]
fn declarative_control_popup_effect_routes_to_resolved_trigger() {
    let mut surface = control_anchored_declarative_popup_surface();
    let result = surface.apply_dispatch_reply(
        keyboard_event("Enter", 13),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::Popup {
            kind: UiPopupEffectKind::Open,
            popup_id: "root/popup".to_string(),
            owner: Some(node(2)),
            anchor: None,
        }),
    );

    assert!(result.rejected_effects.is_empty());
    assert_eq!(result.diagnostics.route_target, Some(node(3)));
    assert_popup_open(&surface, node(2), true);
    assert_eq!(surface.input.popup_stack.len(), 1);
    assert_eq!(surface.input.popup_stack[0].popup_node, Some(node(2)));
    assert_eq!(surface.input.popup_stack[0].owner, Some(node(3)));
}

#[test]
fn declarative_control_popup_close_reroutes_after_trigger_rebind() {
    let mut surface = control_anchored_declarative_popup_surface();
    let opened = surface.apply_dispatch_reply(
        keyboard_event("Enter", 13),
        popup_effect(UiPopupEffectKind::Open),
    );
    assert_eq!(opened.diagnostics.route_target, Some(node(3)));

    set_popup_control_anchor(&mut surface, "replacement-trigger");
    insert_control_trigger(&mut surface, node(4), "replacement-trigger");

    let closed = surface.apply_dispatch_reply(
        keyboard_event("Escape", 27),
        popup_effect(UiPopupEffectKind::Close),
    );

    assert!(closed.rejected_effects.is_empty());
    assert_eq!(closed.diagnostics.route_target, Some(node(4)));
    assert_popup_open(&surface, node(2), false);
    assert_popup_stack(&surface, &[]);
}

#[test]
fn declarative_control_popup_close_drops_invalid_trigger_route() {
    let mut surface = control_anchored_declarative_popup_surface();
    let opened = surface.apply_dispatch_reply(
        keyboard_event("Enter", 13),
        popup_effect(UiPopupEffectKind::Open),
    );
    assert_eq!(opened.diagnostics.route_target, Some(node(3)));

    surface.tree.node_mut(node(3)).unwrap().state_flags.enabled = false;

    let closed = surface.apply_dispatch_reply(
        keyboard_event("Escape", 27),
        popup_effect(UiPopupEffectKind::Close),
    );

    assert!(closed.rejected_effects.is_empty());
    assert_eq!(closed.diagnostics.route_target, None);
    assert_popup_open(&surface, node(2), false);
    assert_popup_stack(&surface, &[]);
}

#[test]
fn declarative_popup_close_resolves_conflicting_open_aliases() {
    let mut surface = declarative_popup_surface(false);
    surface
        .tree
        .node_mut(node(2))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .attributes
        .insert("open".to_string(), toml::Value::Boolean(true));

    let report = surface
        .mutate_property(UiPropertyMutationRequest::new(
            node(2),
            "popup_open",
            UiValue::Bool(false),
        ))
        .unwrap();

    assert!(matches!(report.status, UiPropertyMutationStatus::Accepted));
    assert_popup_open(&surface, node(2), false);
    assert_popup_open_alias(&surface, node(2), false);
    surface.rebuild();
    assert_popup_stack(&surface, &[]);
}

fn declarative_popup_surface(popup_open: bool) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.popup.declarative-state"));
    surface.tree.insert_root(
        UiTreeNode::new(node(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 180.0, 110.0)),
    );
    insert_popup(&mut surface, node(1), node(2), "root/popup", popup_open);
    surface.rebuild();
    surface
}

fn control_anchored_declarative_popup_surface() -> UiSurface {
    let mut surface = declarative_popup_surface(false);
    set_popup_control_anchor(&mut surface, "runtime-popup-trigger");
    insert_control_trigger(&mut surface, node(3), "runtime-popup-trigger");
    surface.rebuild();
    surface
}

fn set_popup_control_anchor(surface: &mut UiSurface, control_id: &str) {
    surface
        .tree
        .node_mut(node(2))
        .unwrap()
        .template_metadata
        .as_mut()
        .unwrap()
        .widget
        .popup_anchor = UiPopupAnchor::Control {
        control_id: control_id.to_string(),
    };
}

fn insert_control_trigger(surface: &mut UiSurface, node_id: UiNodeId, control_id: &str) {
    surface
        .tree
        .insert_child(
            node(1),
            UiTreeNode::new(
                node_id,
                UiNodePath::new(format!("root/trigger/{}", node_id.0)),
            )
            .with_frame(UiFrame::new(96.0, 6.0, 24.0, 20.0))
            .with_input_policy(UiInputPolicy::Receive)
            .with_state_flags(container_state())
            .with_template_metadata(UiTemplateNodeMetadata {
                component: "Button".to_string(),
                control_id: Some(control_id.to_string()),
                widget: UiWidgetContract {
                    behavior: UiWidgetBehavior::Button,
                    ..UiWidgetContract::default()
                },
                ..UiTemplateNodeMetadata::default()
            }),
        )
        .unwrap();
}

fn insert_nested_popup(surface: &mut UiSurface) {
    insert_popup(surface, node(2), node(4), "root/popup/nested", true);
    surface.rebuild();
}

fn insert_deep_nested_popup(surface: &mut UiSurface) {
    insert_nested_popup(surface);
    insert_popup(surface, node(4), node(5), "root/popup/nested/deep", true);
    surface.rebuild();
}

fn insert_popup(
    surface: &mut UiSurface,
    parent_id: UiNodeId,
    node_id: UiNodeId,
    path: &str,
    popup_open: bool,
) {
    surface
        .tree
        .insert_child(
            parent_id,
            UiTreeNode::new(node_id, UiNodePath::new(path))
                .with_frame(UiFrame::new(8.0, 8.0, 120.0, 64.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(container_state())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "MenuPopup".to_string(),
                    attributes: [("popup_open".to_string(), toml::Value::Boolean(popup_open))]
                        .into_iter()
                        .collect(),
                    widget: UiWidgetContract {
                        behavior: UiWidgetBehavior::Popup,
                        open_property: Some("popup_open".to_string()),
                        ..UiWidgetContract::default()
                    },
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
}

fn popup_effect(kind: UiPopupEffectKind) -> UiDispatchReply {
    UiDispatchReply::handled().with_effect(UiDispatchEffect::Popup {
        kind,
        popup_id: "root/popup".to_string(),
        owner: None,
        anchor: None,
    })
}

fn assert_popup_open(surface: &UiSurface, node_id: UiNodeId, expected: bool) {
    let metadata = surface
        .tree
        .node(node_id)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["popup_open"].as_bool(), Some(expected));
}

fn assert_popup_open_alias(surface: &UiSurface, node_id: UiNodeId, expected: bool) {
    let metadata = surface
        .tree
        .node(node_id)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap();
    assert_eq!(metadata.attributes["open"].as_bool(), Some(expected));
}

fn assert_popup_stack(surface: &UiSurface, expected: &[&str]) {
    assert_eq!(
        surface
            .input
            .popup_stack
            .iter()
            .map(|popup| popup.popup_id.as_str())
            .collect::<Vec<_>>(),
        expected
    );
}

fn container_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        ..UiStateFlags::default()
    }
}

fn keyboard_event(logical_key: &str, key_code: u32) -> UiInputEvent {
    UiInputEvent::Keyboard(UiKeyboardInputEvent {
        metadata: UiInputEventMetadata::new(
            UiInputTimestamp::from_micros(40),
            UiInputSequence::new(4),
        ),
        state: UiKeyboardInputState::Pressed,
        key_code,
        scan_code: None,
        physical_key: logical_key.to_string(),
        logical_key: logical_key.to_string(),
        text: None,
    })
}

fn node(id: u64) -> UiNodeId {
    UiNodeId::new(id)
}
