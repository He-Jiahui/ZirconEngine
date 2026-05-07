use super::super::support::*;

#[test]
fn shared_viewport_pointer_bridge_maps_secondary_button_to_right_pressed_event() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_shared_pointer_secondary");
    let mut bridge = SharedViewportPointerBridge::new(UiFrame::new(0.0, 0.0, 320.0, 180.0));

    let effects = dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(24.0, 32.0))
            .with_button(UiPointerButton::Secondary),
    )
    .unwrap();

    let journal = harness.runtime.journal();
    let record = journal.records().last().unwrap();
    assert_eq!(
        record.event,
        EditorEvent::Viewport(EditorViewportEvent::RightPressed { x: 24.0, y: 32.0 })
    );
    assert!(effects.presentation_dirty);
    assert!(effects.render_dirty);
}

#[test]
fn shared_viewport_pointer_bridge_keeps_move_and_up_routed_to_captured_viewport() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_shared_pointer_capture");
    let mut bridge = SharedViewportPointerBridge::new(UiFrame::new(0.0, 0.0, 100.0, 100.0));

    dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(10.0, 10.0))
            .with_button(UiPointerButton::Primary),
    )
    .unwrap();
    dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(180.0, 180.0)),
    )
    .unwrap();
    dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(180.0, 180.0))
            .with_button(UiPointerButton::Primary),
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
            EditorEvent::Viewport(EditorViewportEvent::LeftPressed { x: 10.0, y: 10.0 }),
            EditorEvent::Viewport(EditorViewportEvent::PointerMoved { x: 180.0, y: 180.0 }),
            EditorEvent::Viewport(EditorViewportEvent::LeftReleased),
        ]
    );
}

#[test]
fn shared_viewport_pointer_bridge_maps_scroll_to_viewport_scrolled_event() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_shared_pointer_scroll");
    let mut bridge = SharedViewportPointerBridge::new(UiFrame::new(0.0, 0.0, 160.0, 90.0));

    let effects = dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Scroll, UiPoint::new(32.0, 24.0))
            .with_scroll_delta(-48.0),
    )
    .unwrap();

    let journal = harness.runtime.journal();
    let record = journal.records().last().unwrap();
    assert_eq!(
        record.event,
        EditorEvent::Viewport(EditorViewportEvent::Scrolled { delta: -48.0 })
    );
    assert!(effects.presentation_dirty);
    assert!(effects.render_dirty);
}

#[test]
fn shared_viewport_pointer_bridge_respects_updated_viewport_frame_bounds() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_shared_pointer_frame");
    let mut bridge = SharedViewportPointerBridge::new(UiFrame::new(0.0, 0.0, 160.0, 90.0));
    bridge.update_viewport_frame(UiFrame::new(0.0, 0.0, 80.0, 60.0));

    let record_count_before = harness.runtime.journal().records().len();
    let effects = dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(120.0, 70.0))
            .with_button(UiPointerButton::Primary),
    )
    .unwrap();

    assert_eq!(
        effects,
        crate::ui::slint_host::event_bridge::UiHostEventEffects::default()
    );
    assert_eq!(
        harness.runtime.journal().records().len(),
        record_count_before
    );
}
