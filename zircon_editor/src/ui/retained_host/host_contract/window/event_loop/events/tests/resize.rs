use crate::ui::retained_host::host_contract::redraw::HostRedrawRequest;
use crate::ui::retained_host::primitives::PhysicalSize;
use zircon_runtime_interface::ui::{
    layout::UiSize,
    window::{
        UiWindowEvent, UiWindowEventKind, UiWindowEventMetadata, UiWindowInputPumpEvent,
        UiWindowMetrics, UiWindowPixelSize,
    },
};

use super::{translated_scale_factor, translated_window_metrics, UiHostWindowEventLoop};

#[test]
fn resize_frame_queue_coalesces_to_the_latest_presenter_extent() {
    let host =
        crate::ui::retained_host::host_contract::window::UiHostWindow::new().expect("host window");
    let mut event_loop = UiHostWindowEventLoop::new(host);
    event_loop.pending_redraw = HostRedrawRequest::None;

    event_loop.queue_resize_frame(Some(PhysicalSize::new(800, 600)));
    event_loop.queue_resize_frame(Some(PhysicalSize::new(1440, 900)));

    assert_eq!(event_loop.pending_presenter_resize, Some((1440, 900)));
    assert!(event_loop.pending_redraw.request_redraw());
    assert!(event_loop.pending_redraw.requires_frame_update());
    assert!(event_loop.pending_redraw.prefers_interactive_frame_update());
}

#[test]
fn surface_resize_queues_latest_extent_for_one_interactive_frame_update() {
    let source = include_str!("../resize.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("resize production source");
    let queue = production
        .find("fn queue_resize_frame")
        .expect("resize events must share one frame-bound reflow queue");
    let retain_latest_size = production
        .find("self.pending_presenter_resize = Some")
        .expect("surface resize must retain only the latest presenter size");
    let queue_frame = production
        .find("HostRedrawRequest::full_frame_for_scenario")
        .expect("surface resize must queue an interactive frame update");

    assert!(queue < retain_latest_size);
    assert!(retain_latest_size < queue_frame);
    assert!(!production.contains("presenter.resize"));
    assert!(production.contains("UiPerfScenario::WindowResize"));
    assert!(production.contains("true)"));
    assert!(production.contains("into_interactive_frame_update"));
    assert!(!production.contains("NATIVE_RESIZE_REFLOW_DEBOUNCE"));
}

#[test]
fn translated_surface_resize_preserves_the_prior_scale_factor() {
    let metrics = UiWindowMetrics::new(
        UiSize::new(1280.0, 720.0),
        UiWindowPixelSize::new(2560, 1440),
        2.0,
    );
    let event = UiWindowInputPumpEvent::Window(UiWindowEvent::new(
        UiWindowEventMetadata::default(),
        UiWindowEventKind::Resized { metrics },
    ));

    assert_eq!(translated_window_metrics(Some(event)), Some(metrics));
}

#[test]
fn translated_scale_event_drives_the_retained_window_scale() {
    let event = UiWindowInputPumpEvent::Window(UiWindowEvent::new(
        UiWindowEventMetadata::default(),
        UiWindowEventKind::ScaleFactorChanged { scale_factor: 2.0 },
    ));

    assert_eq!(translated_scale_factor(Some(event)), Some(2.0));
}
