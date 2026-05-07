use crate::ui::template::{UiTemplateInstance, UiTemplateLoader, UiTemplateSurfaceBuilder};
use crate::ui::tree::UiRuntimeTreeAccessExt;
use crate::ui::{
    dispatch::{UiNavigationDispatcher, UiPointerDispatcher},
    surface::UiSurface,
};
use zircon_runtime_interface::ui::dispatch::{
    UiDispatchDisposition, UiDispatchEffect, UiDispatchHostRequestKind, UiDispatchReply,
    UiFocusEffectReason, UiImeInputEvent, UiImeInputEventKind, UiInputEvent, UiInputEventMetadata,
    UiInputMethodRequest, UiInputMethodRequestKind, UiInputSequence, UiInputTimestamp,
    UiKeyboardInputEvent, UiKeyboardInputState, UiNavigationRequestPolicy, UiPointerCaptureReason,
    UiPointerComponentEventReason, UiPointerDispatchEffect, UiPointerEvent, UiPointerId,
    UiPointerInputEvent, UiPointerLockPolicy, UiPreciseScrollDelta, UiTextInputEvent,
};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::{UiComponentEvent, UiComponentEventKind, UiValue},
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{
        AxisConstraint, BoxConstraints, StretchMode, UiAxis, UiContainerKind, UiFrame, UiPoint,
        UiScrollState, UiScrollableBoxConfig, UiScrollbarVisibility, UiSize, UiVirtualListConfig,
    },
    surface::{
        UiHitTestQuery, UiNavigationEventKind, UiPointerButton, UiPointerEventKind,
        UiVirtualPointerPosition,
    },
    template::UiBindingRef,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode, UiVisibility},
};

#[test]
fn primary_release_inside_pressed_target_marks_click_target_and_clears_press_state() {
    let mut surface = button_surface();

    let down = surface
        .route_pointer_event_with_button(
            UiPointerEventKind::Down,
            UiPoint::new(20.0, 20.0),
            UiPointerButton::Primary,
        )
        .unwrap();

    assert_eq!(down.target, Some(UiNodeId::new(2)));
    assert_eq!(down.pressed, Some(UiNodeId::new(2)));
    assert_eq!(down.click_target, None);
    assert_eq!(surface.focus.pressed, Some(UiNodeId::new(2)));

    let up = surface
        .route_pointer_event_with_button(
            UiPointerEventKind::Up,
            UiPoint::new(20.0, 20.0),
            UiPointerButton::Primary,
        )
        .unwrap();

    assert_eq!(up.pressed, Some(UiNodeId::new(2)));
    assert_eq!(up.click_target, Some(UiNodeId::new(2)));
    assert!(up.release_inside_pressed);
    assert_eq!(surface.focus.pressed, None);
}

#[test]
fn primary_release_outside_pressed_target_does_not_mark_click_target() {
    let mut surface = button_surface();

    surface
        .route_pointer_event_with_button(
            UiPointerEventKind::Down,
            UiPoint::new(20.0, 20.0),
            UiPointerButton::Primary,
        )
        .unwrap();
    let up = surface
        .route_pointer_event_with_button(
            UiPointerEventKind::Up,
            UiPoint::new(140.0, 80.0),
            UiPointerButton::Primary,
        )
        .unwrap();

    assert_eq!(up.pressed, Some(UiNodeId::new(2)));
    assert_eq!(up.click_target, None);
    assert!(!up.release_inside_pressed);
    assert_eq!(surface.focus.pressed, None);
}

#[test]
fn captured_release_uses_hit_path_not_capture_target_for_click_target() {
    let mut surface = button_surface();
    surface.focus.pressed = Some(UiNodeId::new(2));
    surface.focus.captured = Some(UiNodeId::new(2));

    let up = surface
        .route_pointer_event_with_button(
            UiPointerEventKind::Up,
            UiPoint::new(140.0, 80.0),
            UiPointerButton::Primary,
        )
        .unwrap();

    assert_eq!(up.target, Some(UiNodeId::new(2)));
    assert_eq!(up.hit_path.target, None);
    assert_eq!(up.click_target, None);
    assert!(!up.release_inside_pressed);
    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.focus.pressed, None);
}

#[test]
fn pointer_route_can_use_virtual_pointer_hit_from_custom_surface_mapper() {
    let mut surface = button_surface();
    let virtual_pointer =
        UiVirtualPointerPosition::new(UiPoint::new(20.0, 20.0), UiPoint::new(18.0, 18.0));
    let route = surface
        .route_pointer_event_with_query_and_button(
            UiPointerEventKind::Down,
            UiHitTestQuery::new(UiPoint::new(140.0, 80.0)).with_virtual_pointer(virtual_pointer),
            UiPointerButton::Primary,
        )
        .unwrap();

    assert_eq!(route.point, UiPoint::new(20.0, 20.0));
    assert_eq!(route.target, Some(UiNodeId::new(2)));
    assert_eq!(route.hit_path.virtual_pointer, Some(virtual_pointer));
    assert_eq!(surface.focus.pressed, Some(UiNodeId::new(2)));
}

#[test]
fn pointer_dispatch_uses_virtual_pointer_query_for_component_events() {
    let mut surface =
        bound_button_surface(vec![binding("Showcase/ButtonPress", UiEventKind::Press)]);
    let virtual_pointer =
        UiVirtualPointerPosition::new(UiPoint::new(20.0, 20.0), UiPoint::new(18.0, 18.0));

    let result = surface
        .dispatch_pointer_event_with_query(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(140.0, 80.0))
                .with_button(UiPointerButton::Primary),
            UiHitTestQuery::new(UiPoint::new(140.0, 80.0)).with_virtual_pointer(virtual_pointer),
        )
        .unwrap();

    assert_eq!(result.route.target, Some(UiNodeId::new(2)));
    assert_eq!(result.route.hit_path.virtual_pointer, Some(virtual_pointer));
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(
        result.component_events[0].binding_id,
        "Showcase/ButtonPress"
    );
}

#[test]
fn pointer_dispatch_result_reports_same_target_hover_as_idle_diagnostic() {
    let mut surface = button_surface();

    let first = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();
    assert!(!first.diagnostics.ignored_same_target_hover);
    assert_eq!(first.diagnostics.hover_entered, 1);

    let second = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(25.0, 25.0)),
        )
        .unwrap();

    assert!(second.diagnostics.pointer_routed);
    assert!(second.diagnostics.ignored_same_target_hover);
    assert_eq!(second.diagnostics.hover_entered, 0);
    assert_eq!(second.diagnostics.hover_left, 0);
}

#[test]
fn repeated_same_target_mouse_moves_do_not_dirty_or_rebuild_surface() {
    let mut surface = button_surface();
    let initial_rebuild = surface.last_rebuild_report;

    let first = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();
    assert_eq!(
        first.requested_damage,
        vec![UiFrame::new(10.0, 10.0, 80.0, 30.0)]
    );

    for offset in 0..100 {
        let point = UiPoint::new(21.0 + (offset % 8) as f32, 21.0);
        let result = surface
            .dispatch_pointer_event(
                &crate::ui::dispatch::UiPointerDispatcher::default(),
                UiPointerEvent::new(UiPointerEventKind::Move, point),
            )
            .unwrap();
        assert!(result.diagnostics.ignored_same_target_hover);
        assert!(result.requested_damage.is_empty());
        assert!(result.component_events.is_empty());
        assert_eq!(surface.last_rebuild_report, initial_rebuild);
        assert!(!surface.dirty_flags().any());
    }
}

#[test]
fn click_component_events_preserve_every_matching_binding_on_target() {
    let mut surface = bound_button_surface(vec![
        binding("Showcase/ButtonPrimary", UiEventKind::Click),
        binding("Showcase/ButtonAudit", UiEventKind::Click),
    ]);

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let up = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(up.component_events.len(), 2);
    assert_eq!(up.component_events[0].binding_id, "Showcase/ButtonPrimary");
    assert_eq!(up.component_events[1].binding_id, "Showcase/ButtonAudit");
    for event in &up.component_events {
        assert_eq!(event.node_id, UiNodeId::new(2));
        assert_eq!(event.event_kind, UiEventKind::Click);
        assert_eq!(event.reason, UiPointerComponentEventReason::DefaultClick);
        assert_eq!(event.envelope.document_id, "runtime.ui.events");
        assert_eq!(event.envelope.control_id, "MaterialButton");
        assert_eq!(event.envelope.event_kind, UiComponentEventKind::Commit);
        assert_eq!(
            event.envelope.event,
            UiComponentEvent::Commit {
                property: "activated".to_string(),
                value: UiValue::Bool(true),
            }
        );
    }
}

#[test]
fn focus_component_events_emit_focus_and_blur_for_matching_bindings() {
    let mut surface = two_button_surface(
        Some(UiTemplateNodeMetadata {
            component: "MaterialButton".to_string(),
            control_id: Some("FirstButton".to_string()),
            bindings: vec![
                binding("Showcase/FirstFocus", UiEventKind::Focus),
                binding("Showcase/FirstBlur", UiEventKind::Blur),
            ],
            ..Default::default()
        }),
        Some(UiTemplateNodeMetadata {
            component: "MaterialButton".to_string(),
            control_id: Some("SecondButton".to_string()),
            bindings: vec![binding("Showcase/SecondFocus", UiEventKind::Focus)],
            ..Default::default()
        }),
    );

    let first_down = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(first_down.component_events.len(), 1);
    assert_eq!(
        first_down.component_events[0].binding_id,
        "Showcase/FirstFocus"
    );
    assert_eq!(
        first_down.component_events[0].event_kind,
        UiEventKind::Focus
    );
    assert_eq!(
        first_down.component_events[0].reason,
        UiPointerComponentEventReason::FocusGained
    );

    let second_down = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 60.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(3)));
    assert_eq!(second_down.component_events.len(), 2);
    assert_eq!(
        second_down.component_events[0].binding_id,
        "Showcase/FirstBlur"
    );
    assert_eq!(
        second_down.component_events[0].event_kind,
        UiEventKind::Blur
    );
    assert_eq!(
        second_down.component_events[0].reason,
        UiPointerComponentEventReason::FocusLost
    );
    assert_eq!(
        second_down.component_events[1].binding_id,
        "Showcase/SecondFocus"
    );
    assert_eq!(
        second_down.component_events[1].event_kind,
        UiEventKind::Focus
    );
    assert_eq!(
        second_down.component_events[1].reason,
        UiPointerComponentEventReason::FocusGained
    );
}

#[test]
fn release_capture_effect_clears_only_current_captor() {
    let mut surface = two_button_surface(None, None);
    surface.focus.captured = Some(UiNodeId::new(2));
    surface.focus.pressed = Some(UiNodeId::new(2));
    let mut dispatcher = crate::ui::dispatch::UiPointerDispatcher::default();
    dispatcher.register(UiNodeId::new(2), UiPointerEventKind::Move, |_context| {
        UiPointerDispatchEffect::release_capture()
    });

    let result = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();

    assert_eq!(surface.focus.captured, None);
    assert_eq!(result.released_capture, Some(UiNodeId::new(2)));
    assert!(result.diagnostics.capture_released);

    surface.focus.captured = Some(UiNodeId::new(3));
    let ignored = surface
        .dispatch_pointer_event(
            &dispatcher,
            UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(20.0, 20.0)),
        )
        .unwrap();

    assert_eq!(surface.focus.captured, Some(UiNodeId::new(3)));
    assert_eq!(ignored.released_capture, None);
    assert!(ignored.invocations.is_empty());
    assert!(!ignored.diagnostics.capture_released);
}

#[test]
fn release_outside_pressed_target_reports_default_click_rejected() {
    let mut surface =
        bound_button_surface(vec![binding("Showcase/ButtonClick", UiEventKind::Click)]);

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(140.0, 80.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(result.route.pressed, Some(UiNodeId::new(2)));
    assert_eq!(result.route.click_target, None);
    assert!(result.diagnostics.default_click_rejected);
    assert!(result.component_events.is_empty());
}

#[test]
fn scroll_fallback_reports_scroll_defaulted_when_unhandled() {
    let mut surface = scrollable_surface();

    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Scroll, UiPoint::new(20.0, 20.0))
                .with_scroll_delta(50.0),
        )
        .unwrap();

    assert_eq!(result.handled_by, Some(UiNodeId::new(2)));
    assert!(result.diagnostics.scroll_defaulted);
}

#[test]
fn scroll_fallback_does_not_handle_when_scroll_offset_is_unchanged() {
    let mut surface = scrollable_surface();

    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Scroll, UiPoint::new(20.0, 20.0))
                .with_scroll_delta(0.0),
        )
        .unwrap();

    assert_eq!(result.handled_by, None);
    assert!(!result.diagnostics.scroll_defaulted);
}

#[test]
fn scroll_fallback_continues_to_ancestor_when_nearest_scrollable_is_clamped() {
    let mut surface = nested_scrollable_surface();

    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Scroll, UiPoint::new(20.0, 20.0))
                .with_scroll_delta(20.0),
        )
        .unwrap();

    assert_eq!(result.handled_by, Some(UiNodeId::new(2)));
    assert!(result.diagnostics.scroll_defaulted);
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(2))
            .unwrap()
            .scroll_state
            .unwrap()
            .offset,
        20.0
    );
    assert_eq!(
        surface
            .tree
            .node(UiNodeId::new(3))
            .unwrap()
            .scroll_state
            .unwrap()
            .offset,
        0.0
    );
}

#[test]
fn pointer_dispatch_result_counts_component_events() {
    let mut surface = bound_button_surface(vec![
        binding("Showcase/ButtonPress", UiEventKind::Press),
        binding("Showcase/ButtonFocus", UiEventKind::Focus),
    ]);

    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(20.0, 20.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(result.component_events.len(), 2);
    assert_eq!(result.diagnostics.component_event_count, 2);
    assert_eq!(result.component_events[0].event_kind, UiEventKind::Press);
    assert_eq!(result.component_events[1].event_kind, UiEventKind::Focus);
}

#[test]
fn bound_custom_template_component_dispatches_click_envelope_after_build() {
    let mut surface = template_surface_from_root_toml(root_with_inline_node(
        r#"{ component = "ScriptActionChip", control_id = "ActionChip", bindings = [{ id = "Demo/Action", event = "Click", route = "Demo.Action" }], attributes = { layout = { width = { min = 80.0, preferred = 80.0, max = 80.0, stretch = "Fixed" }, height = { min = 30.0, preferred = 30.0, max = 30.0, stretch = "Fixed" } } } }"#,
    ));
    surface.compute_layout(UiSize::new(100.0, 50.0)).unwrap();

    surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(10.0, 10.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();
    let result = surface
        .dispatch_pointer_event(
            &crate::ui::dispatch::UiPointerDispatcher::default(),
            UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(10.0, 10.0))
                .with_button(UiPointerButton::Primary),
        )
        .unwrap();

    assert_eq!(result.route.click_target, Some(UiNodeId::new(1)));
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].node_id, UiNodeId::new(1));
    assert_eq!(result.component_events[0].envelope.control_id, "ActionChip");
    assert_eq!(result.component_events[0].binding_id, "Demo/Action");
    assert_eq!(result.component_events[0].event_kind, UiEventKind::Click);
    assert_eq!(
        result.component_events[0].reason,
        UiPointerComponentEventReason::DefaultClick
    );
}

#[test]
fn dispatch_reply_applies_focus_capture_high_precision_and_release_effects() {
    let mut surface = button_surface();
    let pointer_id = UiPointerId::new(7);
    let reply = UiDispatchReply::handled().with_effects([
        UiDispatchEffect::SetFocus {
            target: UiNodeId::new(2),
            reason: UiFocusEffectReason::Input,
        },
        UiDispatchEffect::CapturePointer {
            target: UiNodeId::new(2),
            pointer_id,
            reason: UiPointerCaptureReason::Press,
        },
        UiDispatchEffect::UseHighPrecisionPointer {
            target: UiNodeId::new(2),
            enabled: true,
        },
        UiDispatchEffect::ReleasePointerCapture {
            target: UiNodeId::new(2),
            pointer_id,
            reason: UiPointerCaptureReason::Cancel,
        },
    ]);

    let result = surface.apply_dispatch_reply(keyboard_event(), reply);

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.input.captured_pointer_id, None);
    assert_eq!(surface.input.high_precision_owner, None);
    assert_eq!(result.applied_effects.len(), 4);
    assert!(result.rejected_effects.is_empty());

    surface.focus.captured = Some(UiNodeId::new(2));
    surface.input.high_precision_owner = Some(UiNodeId::new(2));
    let stale_release = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::ReleasePointerCapture {
            target: UiNodeId::new(2),
            pointer_id,
            reason: UiPointerCaptureReason::Cancel,
        }),
    );

    assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)));
    assert_eq!(surface.input.high_precision_owner, Some(UiNodeId::new(2)));
    assert!(stale_release.applied_effects.is_empty());
    assert_eq!(stale_release.rejected_effects.len(), 1);
    assert_eq!(
        stale_release.rejected_effects[0].reason,
        "pointer capture belongs to a different or unknown pointer"
    );

    let release_reply = UiDispatchReply::handled().with_effects([
        UiDispatchEffect::CapturePointer {
            target: UiNodeId::new(2),
            pointer_id,
            reason: UiPointerCaptureReason::Press,
        },
        UiDispatchEffect::UseHighPrecisionPointer {
            target: UiNodeId::new(2),
            enabled: true,
        },
        UiDispatchEffect::ReleasePointerCapture {
            target: UiNodeId::new(2),
            pointer_id,
            reason: UiPointerCaptureReason::Cancel,
        },
    ]);

    let release_result = surface.apply_dispatch_reply(keyboard_event(), release_reply);

    assert_eq!(surface.focus.captured, None);
    assert_eq!(surface.input.captured_pointer_id, None);
    assert_eq!(surface.input.high_precision_owner, None);
    assert_eq!(release_result.applied_effects.len(), 3);
    assert!(release_result.rejected_effects.is_empty());

    surface.input.high_precision_owner = Some(UiNodeId::new(2));
    let stale_high_precision_disable = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::UseHighPrecisionPointer {
            target: UiNodeId::new(1),
            enabled: false,
        }),
    );

    assert_eq!(surface.input.high_precision_owner, Some(UiNodeId::new(2)));
    assert!(stale_high_precision_disable.host_requests.is_empty());
    assert_eq!(stale_high_precision_disable.rejected_effects.len(), 1);
    assert_eq!(
        stale_high_precision_disable.rejected_effects[0].reason,
        "high precision owner mismatch"
    );

    surface.focus.captured = Some(UiNodeId::new(1));
    surface.input.captured_pointer_id = Some(pointer_id);
    surface.input.high_precision_owner = Some(UiNodeId::new(1));
    let stale_capture_release = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::ReleasePointerCapture {
            target: UiNodeId::new(2),
            pointer_id,
            reason: UiPointerCaptureReason::Cancel,
        }),
    );

    assert_eq!(surface.focus.captured, Some(UiNodeId::new(1)));
    assert_eq!(surface.input.high_precision_owner, Some(UiNodeId::new(1)));
    assert!(stale_capture_release.applied_effects.is_empty());
    assert_eq!(stale_capture_release.rejected_effects.len(), 1);

    surface.input.pointer_lock_owner = Some(UiNodeId::new(2));
    surface.input.pointer_lock_policy = Some(UiPointerLockPolicy::RawDelta);
    let stale_unlock = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::UnlockPointer {
            target: UiNodeId::new(1),
            policy: UiPointerLockPolicy::RawDelta,
        }),
    );

    assert_eq!(surface.input.pointer_lock_owner, Some(UiNodeId::new(2)));
    assert!(stale_unlock.host_requests.is_empty());
    assert_eq!(
        stale_unlock.rejected_effects[0].reason,
        "pointer lock owner mismatch"
    );
}

#[test]
fn focus_effects_clear_only_their_current_input_owner() {
    let mut surface = two_button_surface(None, None);
    surface.focus_node(UiNodeId::new(2)).unwrap();
    surface.input.input_method_owner = Some(UiNodeId::new(2));

    let stale_clear = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::ClearFocus {
            target: UiNodeId::new(3),
            reason: UiFocusEffectReason::Dismissal,
        }),
    );

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(surface.input.input_method_owner, Some(UiNodeId::new(2)));
    assert!(stale_clear.applied_effects.is_empty());
    assert_eq!(
        stale_clear.rejected_effects[0].reason,
        "focus owner mismatch"
    );

    let clear = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::ClearFocus {
            target: UiNodeId::new(2),
            reason: UiFocusEffectReason::Dismissal,
        }),
    );

    assert_eq!(surface.focus.focused, None);
    assert_eq!(surface.input.input_method_owner, None);
    assert!(clear.rejected_effects.is_empty());

    surface.focus_node(UiNodeId::new(2)).unwrap();
    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let focus_change = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::SetFocus {
            target: UiNodeId::new(3),
            reason: UiFocusEffectReason::Input,
        }),
    );

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(3)));
    assert_eq!(surface.input.input_method_owner, None);
    assert!(focus_change.rejected_effects.is_empty());
}

#[test]
fn dispatch_reply_applies_navigation_and_host_owned_input_effects() {
    let mut surface = two_button_surface(None, None);
    surface.focus_node(UiNodeId::new(2)).unwrap();
    let request = UiInputMethodRequest {
        kind: UiInputMethodRequestKind::Enable,
        owner: UiNodeId::new(3),
        cursor_rect: Some(UiFrame::new(10.0, 50.0, 1.0, 20.0)),
        composition_rects: vec![UiFrame::new(10.0, 50.0, 30.0, 20.0)],
    };
    let reply = UiDispatchReply::handled().with_effects([
        UiDispatchEffect::RequestNavigation {
            kind: UiNavigationEventKind::Next,
            policy: UiNavigationRequestPolicy::Wrap,
        },
        UiDispatchEffect::LockPointer {
            target: UiNodeId::new(3),
            policy: UiPointerLockPolicy::RawDelta,
        },
        UiDispatchEffect::RequestInputMethod {
            request: request.clone(),
        },
    ]);

    let result = surface.apply_dispatch_reply(keyboard_event(), reply);

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(3)));
    assert_eq!(surface.input.pointer_lock_owner, Some(UiNodeId::new(3)));
    assert_eq!(surface.input.input_method_owner, Some(UiNodeId::new(3)));
    assert_eq!(surface.input.input_method_request, Some(request));
    assert_eq!(result.host_requests.len(), 2);
    assert!(matches!(
        result.host_requests[0].request,
        UiDispatchHostRequestKind::PointerLock { .. }
    ));
    assert!(matches!(
        result.host_requests[1].request,
        UiDispatchHostRequestKind::InputMethod(_)
    ));

    let disable = UiInputMethodRequest {
        kind: UiInputMethodRequestKind::Disable,
        owner: UiNodeId::new(3),
        cursor_rect: None,
        composition_rects: Vec::new(),
    };
    let disabled = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::RequestInputMethod {
            request: disable.clone(),
        }),
    );

    assert_eq!(surface.input.input_method_owner, None);
    assert_eq!(surface.input.input_method_request, None);
    assert!(matches!(
        &disabled.host_requests[0].request,
        UiDispatchHostRequestKind::InputMethod(request) if request == &disable
    ));

    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let stale_disable = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled()
            .with_effect(UiDispatchEffect::RequestInputMethod { request: disable }),
    );

    assert_eq!(surface.input.input_method_owner, Some(UiNodeId::new(2)));
    assert!(stale_disable.host_requests.is_empty());
    assert_eq!(stale_disable.rejected_effects.len(), 1);
    assert_eq!(
        stale_disable.rejected_effects[0].reason,
        "input method owner mismatch"
    );

    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(3))
        .unwrap()
        .state_flags
        .enabled = false;
    let invalid_enable = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::RequestInputMethod {
            request: UiInputMethodRequest {
                kind: UiInputMethodRequestKind::Enable,
                owner: UiNodeId::new(3),
                cursor_rect: None,
                composition_rects: Vec::new(),
            },
        }),
    );

    assert_eq!(surface.input.input_method_owner, Some(UiNodeId::new(2)));
    assert!(invalid_enable.host_requests.is_empty());
    assert_eq!(invalid_enable.rejected_effects.len(), 1);
    assert!(invalid_enable.rejected_effects[0]
        .reason
        .starts_with("invalid input owner"));
}

#[test]
fn shared_input_dispatch_routes_keyboard_text_ime_and_preserves_scroll_diagnostics() {
    let mut surface = button_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();
    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let pointer_dispatcher = UiPointerDispatcher::default();
    let navigation_dispatcher = UiNavigationDispatcher::default();

    let keyboard = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            keyboard_event(),
        )
        .unwrap();
    assert!(keyboard.diagnostics.routed);
    assert_eq!(keyboard.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(keyboard.diagnostics.notes, vec!["focused_route_len=2"]);

    let text = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Text(UiTextInputEvent {
                metadata: input_metadata(),
                text: "commit".to_string(),
            }),
        )
        .unwrap();
    assert_eq!(text.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(text.diagnostics.route_target, Some(UiNodeId::new(2)));

    let ime = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Ime(UiImeInputEvent {
                metadata: input_metadata(),
                kind: UiImeInputEventKind::Cancel,
                text: String::new(),
                cursor_range: None,
            }),
        )
        .unwrap();
    assert_eq!(ime.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(surface.input.input_method_owner, None);
    assert!(ime
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "ime owner cleared"));

    let scroll = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Pointer(UiPointerInputEvent {
                metadata: input_metadata(),
                event: UiPointerEvent::new(UiPointerEventKind::Scroll, UiPoint::new(20.0, 20.0))
                    .with_scroll_delta(-3.5),
                precise_scroll: Some(UiPreciseScrollDelta::pixels(2.25, -3.5)),
            }),
        )
        .unwrap();
    assert!(scroll
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "scroll_delta=-3.5"));
    let UiInputEvent::Pointer(pointer) = scroll.event else {
        panic!("scroll dispatch changed event family");
    };
    assert_eq!(
        pointer.precise_scroll,
        Some(UiPreciseScrollDelta::pixels(2.25, -3.5))
    );

    surface.focus_node(UiNodeId::new(2)).unwrap();
    surface.input.input_method_owner = Some(UiNodeId::new(99));
    let stale_text = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Text(UiTextInputEvent {
                metadata: input_metadata(),
                text: "ignored".to_string(),
            }),
        )
        .unwrap();

    assert_eq!(stale_text.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(stale_text.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(surface.input.input_method_owner, None);
}

#[test]
fn shared_text_input_mutates_focused_editable_value_and_marks_text_dirty() {
    let mut surface = editable_text_surface("Hi", 2);
    surface.focus_node(UiNodeId::new(2)).unwrap();
    let pointer_dispatcher = UiPointerDispatcher::default();
    let navigation_dispatcher = UiNavigationDispatcher::default();

    let result = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Text(UiTextInputEvent {
                metadata: input_metadata(),
                text: "!".to_string(),
            }),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(editable_attr_string(&surface, "value"), "Hi!");
    assert_eq!(editable_attr_usize(&surface, "caret_offset"), 3);
    let node = surface.tree.nodes.get(&UiNodeId::new(2)).unwrap();
    assert!(node.dirty.layout);
    assert!(node.dirty.render);
    assert!(node.dirty.text);
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note.starts_with("text_property_changed:value:")));
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "value".to_string(),
            value: UiValue::String("Hi!".to_string()),
        }
    );
}

#[test]
fn shared_ime_preedit_commit_and_cancel_mutate_editable_composition() {
    let mut surface = editable_text_surface("", 0);
    surface.focus_node(UiNodeId::new(2)).unwrap();
    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let pointer_dispatcher = UiPointerDispatcher::default();
    let navigation_dispatcher = UiNavigationDispatcher::default();
    let preedit = "拼";

    let preedit_result = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Ime(UiImeInputEvent {
                metadata: input_metadata(),
                kind: UiImeInputEventKind::Preedit,
                text: preedit.to_string(),
                cursor_range: Some(
                    zircon_runtime_interface::ui::dispatch::UiTextByteRange::new(
                        preedit.len() as u32,
                        preedit.len() as u32,
                    ),
                ),
            }),
        )
        .unwrap();

    assert_eq!(
        preedit_result.reply.disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(editable_attr_string(&surface, "value"), preedit);
    assert_eq!(editable_attr_usize(&surface, "composition_start"), 0);
    assert_eq!(
        editable_attr_usize(&surface, "composition_end"),
        preedit.len()
    );
    assert_eq!(editable_attr_string(&surface, "composition_text"), preedit);
    assert_eq!(
        editable_attr_string(&surface, "composition_restore_text"),
        ""
    );
    assert_eq!(editable_attr_usize(&surface, "caret_offset"), preedit.len());

    let cancel_result = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Ime(UiImeInputEvent {
                metadata: input_metadata(),
                kind: UiImeInputEventKind::Cancel,
                text: String::new(),
                cursor_range: None,
            }),
        )
        .unwrap();

    assert_eq!(
        cancel_result.reply.disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(editable_attr_string(&surface, "value"), "");
    assert_eq!(editable_attr_string(&surface, "composition_text"), "");
    assert_eq!(surface.input.input_method_owner, None);

    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let _ = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Ime(UiImeInputEvent {
                metadata: input_metadata(),
                kind: UiImeInputEventKind::Preedit,
                text: preedit.to_string(),
                cursor_range: None,
            }),
        )
        .unwrap();
    let commit_result = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Ime(UiImeInputEvent {
                metadata: input_metadata(),
                kind: UiImeInputEventKind::Commit,
                text: preedit.to_string(),
                cursor_range: None,
            }),
        )
        .unwrap();

    assert_eq!(
        commit_result.reply.disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(editable_attr_string(&surface, "value"), preedit);
    assert_eq!(editable_attr_string(&surface, "composition_text"), "");
    assert_eq!(surface.input.input_method_owner, Some(UiNodeId::new(2)));
    assert!(commit_result.component_events.iter().any(|event| {
        event.event
            == UiComponentEvent::Commit {
                property: "value".to_string(),
                value: UiValue::String(preedit.to_string()),
            }
    }));
}

#[test]
fn shared_input_dispatch_rejects_invalid_owners_and_hidden_ancestors() {
    let mut surface = two_button_surface(None, None);
    let pointer_dispatcher = UiPointerDispatcher::default();
    let navigation_dispatcher = UiNavigationDispatcher::default();

    surface.input.input_method_owner = Some(UiNodeId::new(99));
    let missing_ime_owner = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Ime(UiImeInputEvent {
                metadata: input_metadata(),
                kind: UiImeInputEventKind::Preedit,
                text: "draft".to_string(),
                cursor_range: None,
            }),
        )
        .unwrap();

    assert_eq!(
        missing_ime_owner.reply.disposition,
        UiDispatchDisposition::Unhandled
    );
    assert_eq!(missing_ime_owner.diagnostics.route_target, None);
    assert_eq!(surface.input.input_method_owner, None);
    assert!(missing_ime_owner
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "owner route rejected"));

    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(1))
        .unwrap()
        .visibility = UiVisibility::Collapsed;
    surface.input.input_method_owner = Some(UiNodeId::new(2));
    let hidden_ancestor_text = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            UiInputEvent::Text(UiTextInputEvent {
                metadata: input_metadata(),
                text: "ignored".to_string(),
            }),
        )
        .unwrap();

    assert_eq!(
        hidden_ancestor_text.reply.disposition,
        UiDispatchDisposition::Unhandled
    );
    assert_eq!(hidden_ancestor_text.diagnostics.route_target, None);
    assert_eq!(surface.input.input_method_owner, None);
}

fn button_surface() -> UiSurface {
    button_surface_with_metadata(None)
}

fn bound_button_surface(bindings: Vec<UiBindingRef>) -> UiSurface {
    button_surface_with_metadata(Some(UiTemplateNodeMetadata {
        component: "MaterialButton".to_string(),
        control_id: Some("MaterialButton".to_string()),
        bindings,
        ..Default::default()
    }))
}

fn two_button_surface(
    first_metadata: Option<UiTemplateNodeMetadata>,
    second_metadata: Option<UiTemplateNodeMetadata>,
) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.events"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 100.0)),
    );
    let mut first = UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/first"))
        .with_frame(UiFrame::new(10.0, 10.0, 80.0, 30.0))
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(pointer_state());
    if let Some(metadata) = first_metadata {
        first = first.with_template_metadata(metadata);
    }
    surface.tree.insert_child(UiNodeId::new(1), first).unwrap();
    let mut second = UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/second"))
        .with_frame(UiFrame::new(10.0, 50.0, 80.0, 30.0))
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(pointer_state());
    if let Some(metadata) = second_metadata {
        second = second.with_template_metadata(metadata);
    }
    surface.tree.insert_child(UiNodeId::new(1), second).unwrap();
    surface.rebuild();
    surface
}

fn scrollable_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.events"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_constraints(
            BoxConstraints {
                width: stretch_constraint(0.0, 0.0, 100, 1.0),
                height: stretch_constraint(0.0, 0.0, 100, 1.0),
            },
        ),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/scroll"))
                .with_constraints(BoxConstraints {
                    width: stretch_constraint(200.0, 200.0, 100, 1.0),
                    height: stretch_constraint(90.0, 90.0, 100, 1.0),
                })
                .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Vertical,
                    gap: 0.0,
                    scrollbar_visibility: UiScrollbarVisibility::Auto,
                    virtualization: Some(UiVirtualListConfig {
                        item_extent: 40.0,
                        overscan: 0,
                    }),
                }))
                .with_scroll_state(UiScrollState::default())
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    for item in 0..4 {
        surface
            .tree
            .insert_child(
                UiNodeId::new(2),
                UiTreeNode::new(
                    UiNodeId::new(20 + item),
                    UiNodePath::new(format!("root/scroll/item_{item}")),
                )
                .with_constraints(BoxConstraints {
                    width: stretch_constraint(200.0, 200.0, 100, 1.0),
                    height: fixed_constraint(40.0),
                })
                .with_state_flags(pointer_state()),
            )
            .unwrap();
    }
    surface.compute_layout(UiSize::new(200.0, 90.0)).unwrap();
    surface
}

fn nested_scrollable_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.events"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 200.0, 200.0)),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/outer"))
                .with_frame(UiFrame::new(0.0, 0.0, 200.0, 200.0))
                .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Vertical,
                    gap: 0.0,
                    scrollbar_visibility: UiScrollbarVisibility::Auto,
                    virtualization: None,
                }))
                .with_scroll_state(UiScrollState {
                    offset: 0.0,
                    viewport_extent: 100.0,
                    content_extent: 200.0,
                })
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(2),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/outer/inner"))
                .with_frame(UiFrame::new(10.0, 10.0, 80.0, 80.0))
                .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    axis: UiAxis::Vertical,
                    gap: 0.0,
                    scrollbar_visibility: UiScrollbarVisibility::Auto,
                    virtualization: None,
                }))
                .with_scroll_state(UiScrollState {
                    offset: 0.0,
                    viewport_extent: 80.0,
                    content_extent: 80.0,
                })
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(pointer_state()),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn button_surface_with_metadata(template_metadata: Option<UiTemplateNodeMetadata>) -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.events"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 100.0)),
    );
    let mut button = UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/button"))
        .with_frame(UiFrame::new(10.0, 10.0, 80.0, 30.0))
        .with_input_policy(UiInputPolicy::Receive)
        .with_state_flags(pointer_state());
    if let Some(template_metadata) = template_metadata {
        button = button.with_template_metadata(template_metadata);
    }
    surface.tree.insert_child(UiNodeId::new(1), button).unwrap();
    surface.rebuild();
    surface
}

fn editable_text_surface(value: &str, caret_offset: usize) -> UiSurface {
    button_surface_with_metadata(Some(UiTemplateNodeMetadata {
        component: "TextField".to_string(),
        control_id: Some("EditableText".to_string()),
        bindings: vec![
            binding("EditableText/Change", UiEventKind::Change),
            binding("EditableText/Submit", UiEventKind::Submit),
        ],
        attributes: toml::from_str(&format!(
            r#"
value = "{}"
caret_offset = {}
editable_text = true
"#,
            value, caret_offset
        ))
        .unwrap(),
        ..Default::default()
    }))
}

fn editable_attr_string(surface: &UiSurface, key: &str) -> String {
    surface
        .tree
        .nodes
        .get(&UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get(key))
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string()
}

fn editable_attr_usize(surface: &UiSurface, key: &str) -> usize {
    surface
        .tree
        .nodes
        .get(&UiNodeId::new(2))
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get(key))
        .and_then(|value| value.as_integer())
        .unwrap_or_default() as usize
}

fn binding(id: &str, event: UiEventKind) -> UiBindingRef {
    UiBindingRef {
        id: id.to_string(),
        event,
        route: Some(id.replace('/', ".")),
        action: None,
        targets: Vec::new(),
    }
}

fn template_surface_from_root_toml(root: String) -> UiSurface {
    let document =
        UiTemplateLoader::load_toml_str(&format!("version = 1\n\n[root]\n{root}")).unwrap();
    let instance = UiTemplateInstance::from_document(&document).unwrap();
    UiTemplateSurfaceBuilder::build_surface(UiTreeId::new("runtime.ui.events"), &instance).unwrap()
}

fn root_with_inline_node(node: &str) -> String {
    format!("template = \"Root\"\n\n[components.Root]\nroot = {node}")
}

fn input_metadata() -> UiInputEventMetadata {
    let mut metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(10), UiInputSequence::new(1));
    metadata.pointer_id = Some(UiPointerId::new(7));
    metadata
}

fn keyboard_event() -> UiInputEvent {
    UiInputEvent::Keyboard(UiKeyboardInputEvent {
        metadata: input_metadata(),
        state: UiKeyboardInputState::Pressed,
        key_code: 65,
        scan_code: Some(30),
        physical_key: "KeyA".to_string(),
        logical_key: "A".to_string(),
        text: Some("a".to_string()),
    })
}

fn pointer_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: true,
        pressed: false,
        checked: false,
        dirty: false,
    }
}

fn stretch_constraint(min: f32, preferred: f32, priority: i32, weight: f32) -> AxisConstraint {
    AxisConstraint {
        min,
        max: -1.0,
        preferred,
        priority,
        weight,
        stretch_mode: StretchMode::Stretch,
    }
}

fn fixed_constraint(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size,
        max: size,
        preferred: size,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}
