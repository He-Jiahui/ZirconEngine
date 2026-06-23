use super::*;

#[test]
fn input_manager_window_batch_aggregates_results_and_redraw_requests() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.input_manager"));
    let mut manager = UiInputManager::default();
    let mut batch = UiWindowInputPumpBatch::default();
    batch.push(UiWindowInputPumpEvent::Window(UiWindowEvent::size_changed(
        window_metadata(1),
        UiWindowMetrics::new(
            UiSize::new(320.0, 180.0),
            UiWindowPixelSize::new(640, 360),
            2.0,
        ),
    )));
    batch.push(UiWindowInputPumpEvent::Window(
        UiWindowEvent::request_redraw(window_metadata(2), UiWindowRedrawReason::Animation),
    ));

    let outcome = surface
        .dispatch_window_input_pump_batch(&mut manager, batch)
        .unwrap();

    assert_eq!(outcome.results.len(), 2);
    assert!(outcome.redraw_requested);
    assert!(outcome.host_requests.is_empty());
    assert!(surface.window_state.redraw_requested);
}

#[test]
fn input_manager_tick_records_timer_owner_state() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.input_manager.tick"));
    let mut manager = UiInputManager::default();
    let now = UiInputTimestamp::from_micros(42);

    let injected = manager.tick(&mut surface, now).unwrap();

    assert!(injected.is_empty());
    assert_eq!(manager.timers().last_tick(), Some(now));
}
