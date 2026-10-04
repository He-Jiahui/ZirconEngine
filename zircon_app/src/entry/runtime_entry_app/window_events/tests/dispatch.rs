use super::{window_event_belongs_to_primary, window_event_requests_runtime_frame};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::WindowEvent,
};
use zircon_runtime_interface::ZrRuntimeViewportSizeV1;

#[test]
fn astra_life_a2_window_event_ownership_requires_current_matching_id() {
    let primary = 17_usize;
    let foreign = 29_usize;

    assert!(window_event_belongs_to_primary(Some(primary), primary));
    assert!(!window_event_belongs_to_primary(Some(primary), foreign));
    assert!(!window_event_belongs_to_primary(None, primary));
}

#[test]
fn redraw_delivery_does_not_schedule_another_reactive_frame() {
    assert!(!window_event_requests_runtime_frame(
        &WindowEvent::RedrawRequested,
        ZrRuntimeViewportSizeV1::new(1280, 720),
    ));
}

#[test]
fn unhandled_window_noise_does_not_schedule_a_reactive_frame() {
    assert!(!window_event_requests_runtime_frame(
        &WindowEvent::DragMoved {
            position: PhysicalPosition::new(10.0, 20.0),
        },
        ZrRuntimeViewportSizeV1::new(1280, 720),
    ));
}

#[test]
fn handled_window_events_schedule_frames_but_duplicate_resize_does_not() {
    let viewport_size = ZrRuntimeViewportSizeV1::new(1280, 720);
    assert!(window_event_requests_runtime_frame(
        &WindowEvent::Moved(PhysicalPosition::new(20, 30)),
        viewport_size,
    ));
    assert!(!window_event_requests_runtime_frame(
        &WindowEvent::Focused(false),
        viewport_size,
    ));
    assert!(!window_event_requests_runtime_frame(
        &WindowEvent::Occluded(true),
        viewport_size,
    ));
    assert!(!window_event_requests_runtime_frame(
        &WindowEvent::SurfaceResized(PhysicalSize::new(1280, 720)),
        viewport_size,
    ));
    assert!(window_event_requests_runtime_frame(
        &WindowEvent::SurfaceResized(PhysicalSize::new(1281, 720)),
        viewport_size,
    ));
}

#[test]
fn failed_native_v2_dispatch_keeps_source_ownership_and_suppresses_v1_replay() {
    assert!(native_ime_result_claims_event(Some(false)));
    assert!(native_ime_result_claims_event(Some(true)));
}

#[test]
fn unrequested_native_v2_leaves_the_event_for_the_legacy_v1_path() {
    assert!(!native_ime_result_claims_event(None));
}
