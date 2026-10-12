use super::*;
use crate::ui::retained_host::host_contract::data::FrameRect;
use std::time::Duration;

#[test]
fn redraw_queue_schedules_only_on_empty_to_pending_transition() {
    let host =
        crate::ui::retained_host::host_contract::window::UiHostWindow::new().expect("host window");
    let mut event_loop = UiHostWindowEventLoop::new(host);
    let startup = event_loop.take_pending_redraw();
    assert!(startup.request_redraw());

    assert!(
        event_loop.queue_redraw(HostRedrawRequest::region(FrameRect {
            x: 4.0,
            y: 8.0,
            width: 20.0,
            height: 16.0,
        }))
    );
    assert!(
        !event_loop.queue_redraw(HostRedrawRequest::region(FrameRect {
            x: 40.0,
            y: 48.0,
            width: 12.0,
            height: 10.0,
        }))
    );
}

#[test]
fn native_resize_configures_the_latest_surface_before_the_frame_update() {
    let source = include_str!("../redraw.rs");
    let function = source
        .split("fn redraw_requested_impl")
        .nth(1)
        .and_then(|body| body.split("fn take_pending_redraw").next())
        .expect("redraw implementation");
    let resize = function
        .find("self.apply_pending_presenter_resize(event_loop)")
        .expect("pending swapchain resize should configure the latest size");
    let frame_update = function
        .find("redraw.requires_frame_update()")
        .expect("interactive resize must publish retained geometry");
    let present = function
        .find("present_redraw(")
        .expect("interactive resize should still present the retained snapshot");

    assert!(resize < frame_update);
    assert!(frame_update < present);
    assert!(function.contains("pending_presenter_resize.is_some()"));
    assert!(!function.contains("native_resize_present"));
}

#[test]
fn retryable_surface_present_uses_bounded_exponential_backoff() {
    assert_eq!(surface_present_retry_delay(0), Duration::from_millis(8));
    assert_eq!(surface_present_retry_delay(1), Duration::from_millis(16));
    assert_eq!(surface_present_retry_delay(4), Duration::from_millis(128));
    assert_eq!(surface_present_retry_delay(5), Duration::from_millis(250));
    assert_eq!(
        surface_present_retry_delay(u8::MAX),
        Duration::from_millis(250)
    );
}

#[test]
fn slow_redraw_trace_filters_at_one_hundred_milliseconds_and_caps_logs() {
    assert!(!slow_redraw_trace_is_slow(Duration::from_millis(99)));
    assert!(!slow_redraw_trace_is_slow(Duration::from_millis(100)));
    assert!(slow_redraw_trace_is_slow(Duration::from_millis(101)));

    let counter = AtomicUsize::new(0);
    for _ in 0..SLOW_REDRAW_TRACE_LOG_LIMIT {
        assert!(slow_redraw_trace_slot_available(&counter));
    }
    assert!(!slow_redraw_trace_slot_available(&counter));
    assert_eq!(counter.load(Ordering::Relaxed), SLOW_REDRAW_TRACE_LOG_LIMIT);
}

#[test]
fn retryable_surface_present_is_deferred_outside_the_native_redraw_queue() {
    let host =
        crate::ui::retained_host::host_contract::window::UiHostWindow::new().expect("host window");
    let mut event_loop = UiHostWindowEventLoop::new(host);
    let _startup = event_loop.take_pending_redraw();
    let now = Instant::now();
    let damage = FrameRect {
        x: 4.0,
        y: 8.0,
        width: 32.0,
        height: 16.0,
    };

    event_loop.defer_surface_present_retry(HostRedrawRequest::region(damage.clone()), now);

    assert!(!event_loop.pending_redraw.request_redraw());
    assert_eq!(
        event_loop.pending_surface_present_retry_deadline,
        Some(now + Duration::from_millis(8))
    );
    assert!(!event_loop
        .take_due_surface_present_retry(now + Duration::from_millis(7))
        .request_redraw());
    let retry = event_loop.take_due_surface_present_retry(now + Duration::from_millis(8));
    assert_eq!(retry.damage_region(), Some(&damage));
    assert!(!retry.requires_frame_update());
    assert_eq!(event_loop.pending_surface_present_retry_deadline, None);
}

#[test]
fn real_redraw_consumes_a_deferred_retry_and_success_resets_backoff() {
    let host =
        crate::ui::retained_host::host_contract::window::UiHostWindow::new().expect("host window");
    let mut event_loop = UiHostWindowEventLoop::new(host);
    let _startup = event_loop.take_pending_redraw();
    let now = Instant::now();
    event_loop.defer_surface_present_retry(
        HostRedrawRequest::full_frame_for_scenario(
            crate::ui::retained_host::ui_perf::UiPerfScenario::WindowResize,
            false,
        ),
        now,
    );
    assert!(
        event_loop.queue_redraw(HostRedrawRequest::region(FrameRect {
            x: 10.0,
            y: 12.0,
            width: 8.0,
            height: 6.0,
        }))
    );

    let merged = event_loop.take_redraw_for_present();
    assert!(merged.requires_present());
    assert_eq!(merged.damage_region(), None);
    assert_eq!(event_loop.pending_surface_present_retry_deadline, None);
    assert_eq!(event_loop.surface_present_retry_attempt, 1);

    event_loop.reset_surface_present_retry_backoff();
    assert_eq!(event_loop.surface_present_retry_attempt, 0);
}
