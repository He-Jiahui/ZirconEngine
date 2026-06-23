use super::*;

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
