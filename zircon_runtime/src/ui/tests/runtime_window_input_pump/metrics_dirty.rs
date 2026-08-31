use super::*;

#[test]
fn window_input_pump_resize_updates_frame_metrics_and_layout_dirty_domains() {
    let mut surface = route_surface();
    surface.clear_dirty_flags();
    let metrics = UiWindowMetrics::new(
        UiSize::new(320.0, 180.0),
        UiWindowPixelSize::new(640, 360),
        2.0,
    );

    let result = dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::size_changed(
            window_metadata(18, false),
            metrics,
        )),
    )
    .unwrap();

    assert_eq!(surface.window_state.metrics, Some(metrics));
    assert_eq!(surface.surface_frame().window_state.metrics, Some(metrics));
    assert_eq!(
        surface.dirty_flags(),
        UiDirtyFlags {
            layout: true,
            hit_test: true,
            render: true,
            ..Default::default()
        }
    );
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some(UiDispatchPhase::DefaultAction.as_str())
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_layout_metrics_dirty"));
}

#[test]
fn window_input_pump_scale_factor_updates_retained_metrics_without_losing_size() {
    let mut surface = route_surface();
    let metrics = UiWindowMetrics::new(
        UiSize::new(480.0, 270.0),
        UiWindowPixelSize::new(960, 540),
        2.0,
    );
    dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::size_changed(
            window_metadata(19, false),
            metrics,
        )),
    )
    .unwrap();
    surface.clear_dirty_flags();

    let result = dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::new(
            window_metadata(20, false),
            UiWindowEventKind::ScaleFactorChanged { scale_factor: 1.5 },
        )),
    )
    .unwrap();

    assert_eq!(
        surface.window_state.metrics,
        Some(UiWindowMetrics::new(
            metrics.logical_size,
            metrics.physical_size,
            1.5,
        ))
    );
    assert_eq!(
        surface.dirty_flags(),
        UiDirtyFlags {
            layout: true,
            hit_test: true,
            render: true,
            ..Default::default()
        }
    );
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_scale_factor_updated"));

    surface.rebuild_dirty(metrics.logical_size).unwrap();
    assert_eq!(surface.render_extract.raster_scale, 2.0);

    let settled_metrics =
        UiWindowMetrics::new(metrics.logical_size, UiWindowPixelSize::new(720, 405), 1.5);
    dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::size_changed(
            window_metadata(21, false),
            settled_metrics,
        )),
    )
    .unwrap();
    surface.rebuild_dirty(settled_metrics.logical_size).unwrap();
    assert_eq!(surface.render_extract.raster_scale, 1.5);
}

#[test]
fn window_input_pump_raster_scale_never_undersamples_the_physical_extent() {
    let mut surface = route_surface();
    let metrics = UiWindowMetrics::new(
        UiSize::new(320.0, 180.0),
        UiWindowPixelSize::new(640, 360),
        1.25,
    );

    dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::size_changed(
            window_metadata(22, false),
            metrics,
        )),
    )
    .unwrap();
    surface.rebuild_dirty(metrics.logical_size).unwrap();

    assert_eq!(surface.render_extract.raster_scale, 2.0);
}

#[test]
fn window_input_pump_move_updates_position_without_dirty_domains() {
    let mut surface = route_surface();
    surface.clear_dirty_flags();

    let result = dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::moved_window(
            window_metadata(21, false),
            UiWindowPixelPosition::new(44, 88),
        )),
    )
    .unwrap();

    assert_eq!(
        surface.window_state.position,
        Some(UiWindowPixelPosition::new(44, 88))
    );
    assert_eq!(
        surface.surface_frame().window_state.position,
        Some(UiWindowPixelPosition::new(44, 88))
    );
    assert_eq!(surface.dirty_flags(), UiDirtyFlags::default());
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_position_updated"));
}

#[test]
fn window_input_pump_redraw_request_marks_render_dirty_only() {
    let mut surface = route_surface();
    surface.clear_dirty_flags();

    let result = dispatch_window_input_pump_event(
        &mut surface,
        UiWindowInputPumpEvent::Window(UiWindowEvent::request_redraw(
            window_metadata(22, false),
            UiWindowRedrawReason::Animation,
        )),
    )
    .unwrap();

    assert!(surface.window_state.redraw_requested);
    assert_eq!(surface.window_state.redraw_request_count, 1);
    assert_eq!(
        surface.window_state.last_redraw_reason,
        Some(UiWindowRedrawReason::Animation)
    );
    assert_eq!(
        surface.dirty_flags(),
        UiDirtyFlags {
            render: true,
            ..Default::default()
        }
    );
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_redraw_requested"));
}
