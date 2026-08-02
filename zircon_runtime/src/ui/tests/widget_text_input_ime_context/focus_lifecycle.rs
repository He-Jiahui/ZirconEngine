use super::*;
use crate::ui::surface::UiPropertyMutationRequest;
use zircon_runtime_interface::ui::component::UiValue;

#[test]
fn text_input_focus_enables_ime_and_clear_focus_disables_it() {
    let mut surface = text_input_surface_with_selection("editable", 8, 8, 8);

    let enabled = surface.apply_dispatch_reply(
        input_method_lifecycle_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::SetFocus {
            target: UiNodeId::new(2),
            reason: UiFocusEffectReason::Input,
        }),
    );

    assert_eq!(surface.input.input_method_owner, Some(UiNodeId::new(2)));
    assert!(has_input_method_host_request(
        &enabled,
        UiInputMethodRequestKind::Enable,
        UiNodeId::new(2)
    ));

    let disabled = surface.apply_dispatch_reply(
        input_method_lifecycle_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::ClearFocus {
            target: UiNodeId::new(2),
            reason: UiFocusEffectReason::Dismissal,
        }),
    );

    assert_eq!(surface.input.input_method_owner, None);
    assert!(has_input_method_host_request(
        &disabled,
        UiInputMethodRequestKind::Disable,
        UiNodeId::new(2)
    ));
}

#[test]
fn text_input_focus_loss_commits_active_preedit_before_disabling_ime() {
    let mut surface = focused_text_input_with_active_preedit();

    let result = surface.apply_dispatch_reply(
        input_method_lifecycle_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::ClearFocus {
            target: UiNodeId::new(2),
            reason: UiFocusEffectReason::Dismissal,
        }),
    );

    assert_eq!(text_attr(&surface, "content"), "aXc");
    assert_eq!(text_attr(&surface, "composition_text"), "");
    assert_eq!(surface.input.input_method_owner, None);
    assert!(has_input_method_host_request(
        &result,
        UiInputMethodRequestKind::Disable,
        UiNodeId::new(2)
    ));
    assert!(result.component_events.iter().any(|report| {
        matches!(
            &report.event,
            UiComponentEvent::Commit { property, value }
                if property == "content" && value.display_text() == "aXc"
        )
    }));
}

#[test]
fn secure_text_input_focus_disables_ime_without_exposing_an_owner() {
    let mut surface = text_input_surface_with_selection_and_attributes(
        "password",
        8,
        8,
        8,
        [("secure", toml::Value::Boolean(true))],
    );

    let result = surface.apply_dispatch_reply(
        input_method_lifecycle_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::SetFocus {
            target: UiNodeId::new(2),
            reason: UiFocusEffectReason::Input,
        }),
    );

    assert_eq!(surface.input.input_method_owner, None);
    assert!(has_input_method_host_request(
        &result,
        UiInputMethodRequestKind::Disable,
        UiNodeId::new(2)
    ));
    assert!(!has_input_method_host_request(
        &result,
        UiInputMethodRequestKind::Enable,
        UiNodeId::new(2)
    ));
}

#[test]
fn secure_text_input_rejects_a_direct_ime_enable_request() {
    let mut surface = text_input_surface_with_selection_and_attributes(
        "password",
        8,
        8,
        8,
        [("secure", toml::Value::Boolean(true))],
    );

    let result = surface.apply_dispatch_reply(
        input_method_lifecycle_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::RequestInputMethod {
            request: UiInputMethodRequest {
                kind: UiInputMethodRequestKind::Enable,
                owner: UiNodeId::new(2),
                cursor_rect: None,
                composition_rects: Vec::new(),
                surrounding_text: None,
            },
        }),
    );

    assert_eq!(surface.input.input_method_owner, None);
    assert!(result.host_requests.is_empty());
    assert_eq!(result.rejected_effects.len(), 1);
    assert_eq!(
        result.rejected_effects[0].reason,
        "input method is disabled for secure text input"
    );
}

#[test]
fn modal_redirect_to_a_non_text_focus_target_does_not_enable_the_background_text_input_ime() {
    let mut surface = text_input_surface_with_selection("background", 10, 10, 10);
    add_open_modal_focus_target(&mut surface, false);

    let result = surface.apply_dispatch_reply(
        input_method_lifecycle_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::SetFocus {
            target: UiNodeId::new(2),
            reason: UiFocusEffectReason::Input,
        }),
    );

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(4)));
    assert_eq!(surface.input.input_method_owner, None);
    assert!(!has_input_method_host_request(
        &result,
        UiInputMethodRequestKind::Enable,
        UiNodeId::new(2)
    ));
    assert!(result.host_requests.is_empty());
}

#[test]
fn modal_redirect_to_another_text_input_enables_the_actual_focus_target_ime() {
    let mut surface = text_input_surface_with_selection("background", 10, 10, 10);
    add_open_modal_focus_target(&mut surface, true);

    let result = surface.apply_dispatch_reply(
        input_method_lifecycle_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::SetFocus {
            target: UiNodeId::new(2),
            reason: UiFocusEffectReason::Input,
        }),
    );

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(4)));
    assert_eq!(surface.input.input_method_owner, Some(UiNodeId::new(4)));
    assert!(has_input_method_host_request(
        &result,
        UiInputMethodRequestKind::Enable,
        UiNodeId::new(4)
    ));
    assert!(!has_input_method_host_request(
        &result,
        UiInputMethodRequestKind::Enable,
        UiNodeId::new(2)
    ));
}

#[test]
fn hidden_text_input_commits_preedit_and_disables_ime_on_next_tick() {
    let mut surface = focused_text_input_with_active_preedit();
    let mut manager = UiInputManager::default();
    let focus_change_count = surface.focus.changes.len();

    surface
        .mutate_property(UiPropertyMutationRequest::new(
            UiNodeId::new(2),
            "visible",
            UiValue::Bool(false),
        ))
        .unwrap();

    assert_eq!(surface.focus.focused, None);
    assert_eq!(surface.focus.changes.len(), focus_change_count + 1);
    assert_eq!(text_attr(&surface, "content"), "aXc");
    assert_eq!(text_attr(&surface, "composition_text"), "");
    manager
        .tick(&mut surface, UiInputTimestamp::from_micros(48))
        .unwrap();
    assert_eq!(
        manager.drain_ime_host_requests(),
        vec![ImeHostRequest::Disable]
    );
}

#[test]
fn disabled_text_input_commits_preedit_and_disables_ime_on_next_tick() {
    let mut surface = focused_text_input_with_active_preedit();
    let mut manager = UiInputManager::default();

    surface
        .mutate_property(UiPropertyMutationRequest::new(
            UiNodeId::new(2),
            "enabled",
            UiValue::Bool(false),
        ))
        .unwrap();

    assert_eq!(surface.focus.focused, None);
    assert_eq!(text_attr(&surface, "content"), "aXc");
    assert_eq!(text_attr(&surface, "composition_text"), "");
    manager
        .tick(&mut surface, UiInputTimestamp::from_micros(48))
        .unwrap();
    assert_eq!(
        manager.drain_ime_host_requests(),
        vec![ImeHostRequest::Disable]
    );
}

#[test]
fn detached_text_input_commits_preedit_before_recycling_and_disables_ime() {
    let mut surface = focused_text_input_with_active_preedit();
    let mut manager = UiInputManager::default();

    surface.detach_subtree_to_pool(UiNodeId::new(2)).unwrap();

    assert_eq!(surface.focus.focused, None);
    assert!(surface.tree.node(UiNodeId::new(2)).is_none());
    let results = manager
        .tick(&mut surface, UiInputTimestamp::from_micros(48))
        .unwrap();
    assert_eq!(
        manager.drain_ime_host_requests(),
        vec![ImeHostRequest::Disable]
    );
    assert!(
        results
            .iter()
            .flat_map(|result| &result.component_events)
            .any(|report| {
                matches!(
                    &report.event,
                    UiComponentEvent::Commit { property, value }
                        if property == "content" && value.display_text() == "aXc"
                )
            })
    );
}

#[test]
fn detached_unfocused_ime_owner_commits_preedit_and_disables_ime_on_the_same_tick() {
    let mut surface = unfocused_text_input_with_active_preedit();
    let mut manager = UiInputManager::default();

    surface.detach_subtree_to_pool(UiNodeId::new(2)).unwrap();

    let results = manager
        .tick(&mut surface, UiInputTimestamp::from_micros(48))
        .unwrap();

    assert_eq!(
        manager.drain_ime_host_requests(),
        vec![ImeHostRequest::Disable]
    );
    assert!(
        results
            .iter()
            .flat_map(|result| &result.component_events)
            .any(|report| {
                matches!(
                    &report.event,
                    UiComponentEvent::Commit { property, value }
                        if property == "content" && value.display_text() == "aXc"
                )
            })
    );
}

fn has_input_method_host_request(
    result: &zircon_runtime_interface::ui::dispatch::UiInputDispatchResult,
    kind: UiInputMethodRequestKind,
    owner: UiNodeId,
) -> bool {
    result.host_requests.iter().any(|host_request| {
        matches!(
            &host_request.request,
            UiDispatchHostRequestKind::InputMethod(request)
                if request.kind == kind && request.owner == owner
        )
    })
}

fn input_method_lifecycle_event() -> UiInputEvent {
    UiInputEvent::Ime(UiImeInputEvent {
        metadata: UiInputEventMetadata::new(
            UiInputTimestamp::from_micros(47),
            UiInputSequence::new(13),
        ),
        kind: UiImeInputEventKind::Cancel,
        text: String::new(),
        cursor_range: None,
        delete_surrounding: None,
    })
}

fn add_open_modal_focus_target(surface: &mut UiSurface, target_is_text_input: bool) {
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/modal"))
                .with_frame(UiFrame::new(0.0, 40.0, 180.0, 80.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    ..UiStateFlags::default()
                })
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: "Modal".to_string(),
                    attributes: [("open".to_string(), toml::Value::Boolean(true))]
                        .into_iter()
                        .collect(),
                    ..UiTemplateNodeMetadata::default()
                }),
        )
        .unwrap();
    let metadata = if target_is_text_input {
        UiTemplateNodeMetadata {
            component: "SearchBox".to_string(),
            attributes: [(
                "content".to_string(),
                toml::Value::String("modal".to_string()),
            )]
            .into_iter()
            .collect(),
            widget: UiWidgetContract {
                behavior: UiWidgetBehavior::TextInput,
                value_property: Some("content".to_string()),
                ..UiWidgetContract::default()
            },
            ..UiTemplateNodeMetadata::default()
        }
    } else {
        UiTemplateNodeMetadata {
            component: "Button".to_string(),
            ..UiTemplateNodeMetadata::default()
        }
    };
    surface
        .tree
        .insert_child(
            UiNodeId::new(3),
            UiTreeNode::new(UiNodeId::new(4), UiNodePath::new("root/modal/target"))
                .with_frame(UiFrame::new(8.0, 48.0, 140.0, 28.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(focusable_state())
                .with_template_metadata(metadata),
        )
        .unwrap();
    surface.rebuild();
}

fn focused_text_input_with_active_preedit() -> UiSurface {
    let mut surface = text_input_surface_with_selection("abc", 2, 1, 2);
    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_mut())
        .expect("text input metadata")
        .bindings
        .push(binding("SearchBox/Submit", UiEventKind::Submit));
    surface.focus_node(UiNodeId::new(2)).unwrap();
    let _ = dispatch_ime(&mut surface, UiImeInputEventKind::Preedit, "X", None);
    surface
}

fn unfocused_text_input_with_active_preedit() -> UiSurface {
    let mut surface = text_input_surface_with_selection("abc", 2, 1, 2);
    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_mut())
        .expect("text input metadata")
        .bindings
        .push(binding("SearchBox/Submit", UiEventKind::Submit));
    let _ = surface.apply_dispatch_reply(
        input_method_lifecycle_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::RequestInputMethod {
            request: UiInputMethodRequest {
                kind: UiInputMethodRequestKind::Enable,
                owner: UiNodeId::new(2),
                cursor_rect: None,
                composition_rects: Vec::new(),
                surrounding_text: None,
            },
        }),
    );
    let _ = dispatch_ime(&mut surface, UiImeInputEventKind::Preedit, "X", None);
    surface
}
