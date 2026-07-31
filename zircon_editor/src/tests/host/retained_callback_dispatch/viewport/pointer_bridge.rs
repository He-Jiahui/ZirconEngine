use super::super::support::*;

#[test]
fn shared_viewport_pointer_bridge_maps_secondary_button_to_right_pressed_event() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_shared_pointer_secondary");
    let mut bridge = SharedViewportPointerBridge::new(UiFrame::new(0.0, 0.0, 320.0, 180.0));

    let effects = dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(24.0, 32.0))
            .with_button(UiPointerButton::Secondary),
        Default::default(),
    )
    .unwrap();

    let journal = harness.runtime.journal();
    let record = journal.records().last().unwrap();
    assert_eq!(
        record.event,
        EditorEvent::Viewport(EditorViewportEvent::RightPressed { x: 24.0, y: 32.0 })
    );
    assert!(!effects.presentation_dirty);
    assert!(!effects.render_dirty);
}

#[test]
fn shared_viewport_pointer_bridge_projects_selection_modifiers_on_primary_press() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_shared_pointer_modifiers");
    let mut bridge = SharedViewportPointerBridge::new(UiFrame::new(0.0, 0.0, 320.0, 180.0));

    dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(24.0, 32.0))
            .with_button(UiPointerButton::Primary),
        zircon_runtime_interface::ui::dispatch::UiInputModifiers {
            shift: true,
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Viewport(EditorViewportEvent::LeftPressed {
            x: 24.0,
            y: 32.0,
            selection_mutation: crate::scene::selection::SelectionMutation::Extend,
        })
    );

    dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(24.0, 32.0))
            .with_button(UiPointerButton::Primary),
        Default::default(),
    )
    .unwrap();
    dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(24.0, 32.0))
            .with_button(UiPointerButton::Primary),
        zircon_runtime_interface::ui::dispatch::UiInputModifiers {
            control: true,
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Viewport(EditorViewportEvent::LeftPressed {
            x: 24.0,
            y: 32.0,
            selection_mutation: crate::scene::selection::SelectionMutation::Toggle,
        })
    );
}

#[test]
fn shared_viewport_pointer_bridge_keeps_move_and_up_routed_to_captured_viewport() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_shared_pointer_capture");
    let mut bridge = SharedViewportPointerBridge::new(UiFrame::new(0.0, 0.0, 100.0, 100.0));

    dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(10.0, 10.0))
            .with_button(UiPointerButton::Primary),
        Default::default(),
    )
    .unwrap();
    dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(180.0, 180.0)),
        Default::default(),
    )
    .unwrap();
    dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(180.0, 180.0))
            .with_button(UiPointerButton::Primary),
        Default::default(),
    )
    .unwrap();

    let events: Vec<_> = harness
        .runtime
        .journal()
        .records()
        .iter()
        .rev()
        .take(3)
        .map(|record| record.event.clone())
        .collect();
    assert_eq!(
        events.into_iter().rev().collect::<Vec<_>>(),
        vec![
            EditorEvent::Viewport(EditorViewportEvent::LeftPressed {
                x: 10.0,
                y: 10.0,
                selection_mutation: crate::scene::selection::SelectionMutation::Replace,
            }),
            EditorEvent::Viewport(EditorViewportEvent::PointerMoved { x: 180.0, y: 180.0 }),
            EditorEvent::Viewport(EditorViewportEvent::LeftReleased),
        ]
    );
}

#[test]
fn shared_viewport_pointer_bridge_routes_cancel_to_capture_without_viewport_command() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_shared_pointer_cancel");
    let mut bridge = SharedViewportPointerBridge::new(UiFrame::new(0.0, 0.0, 100.0, 100.0));

    dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(10.0, 10.0))
            .with_button(UiPointerButton::Primary),
        Default::default(),
    )
    .unwrap();
    let record_count_after_down = harness.runtime.journal().records().len();

    let cancel_effects = dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Cancel, UiPoint::new(180.0, 180.0)),
        Default::default(),
    )
    .unwrap();
    assert_eq!(
        cancel_effects,
        crate::ui::retained_host::event_bridge::UiHostEventEffects::default()
    );
    assert_eq!(
        harness.runtime.journal().records().len(),
        record_count_after_down
    );

    dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(180.0, 180.0)),
        Default::default(),
    )
    .unwrap();
    assert_eq!(
        harness.runtime.journal().records().len(),
        record_count_after_down
    );
}

#[test]
fn shared_viewport_pointer_move_without_feedback_stays_dirty_domain_idle() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_shared_pointer_idle_move");
    let mut bridge = SharedViewportPointerBridge::new(UiFrame::new(0.0, 0.0, 100.0, 100.0));

    let effects = dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(24.0, 32.0)),
        Default::default(),
    )
    .unwrap();

    let journal = harness.runtime.journal();
    let record = journal.records().last().unwrap();
    assert_eq!(
        record.event,
        EditorEvent::Viewport(EditorViewportEvent::PointerMoved { x: 24.0, y: 32.0 })
    );
    assert!(!effects.presentation_dirty);
    assert!(!effects.render_dirty);
}

#[test]
fn shared_viewport_pointer_bridge_maps_scroll_to_viewport_scrolled_event() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_shared_pointer_scroll");
    let mut bridge = SharedViewportPointerBridge::new(UiFrame::new(0.0, 0.0, 160.0, 90.0));

    let effects = dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Scroll, UiPoint::new(32.0, 24.0))
            .with_scroll_delta(-48.0),
        Default::default(),
    )
    .unwrap();

    let journal = harness.runtime.journal();
    let record = journal.records().last().unwrap();
    assert_eq!(
        record.event,
        EditorEvent::Viewport(EditorViewportEvent::Scrolled { delta: -48.0 })
    );
    assert!(!effects.presentation_dirty);
    assert!(effects.render_dirty);
}

#[test]
fn shared_viewport_pointer_bridge_respects_updated_viewport_frame_bounds() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_shared_pointer_frame");
    let mut bridge = SharedViewportPointerBridge::new(UiFrame::new(0.0, 0.0, 160.0, 90.0));
    bridge.update_viewport_frame(UiFrame::new(0.0, 0.0, 80.0, 60.0));

    let record_count_before = harness.runtime.journal().records().len();
    let effects = dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(120.0, 70.0))
            .with_button(UiPointerButton::Primary),
        Default::default(),
    )
    .unwrap();

    assert_eq!(
        effects,
        crate::ui::retained_host::event_bridge::UiHostEventEffects::default()
    );
    assert_eq!(
        harness.runtime.journal().records().len(),
        record_count_before
    );
}

#[test]
fn shared_viewport_pointer_bridge_skips_unchanged_frame_rebuilds() {
    let frame = UiFrame::new(0.0, 0.0, 160.0, 90.0);
    let mut bridge = SharedViewportPointerBridge::new(frame);

    assert!(!bridge.update_viewport_frame(frame));
    assert!(bridge.update_viewport_frame(UiFrame::new(0.0, 0.0, 80.0, 60.0)));
    assert!(!bridge.update_viewport_frame(UiFrame::new(0.0, 0.0, 80.0, 60.0)));
}
