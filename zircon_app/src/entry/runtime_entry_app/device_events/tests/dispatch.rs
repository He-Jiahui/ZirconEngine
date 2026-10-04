use winit::event::{DeviceEvent, MouseScrollDelta};

use super::device_event_requests_runtime_frame;

#[test]
fn only_consumed_raw_device_motion_schedules_a_reactive_frame() {
    assert!(device_event_requests_runtime_frame(
        &DeviceEvent::PointerMotion { delta: (1.0, -1.0) }
    ));
    assert!(!device_event_requests_runtime_frame(
        &DeviceEvent::MouseWheel {
            delta: MouseScrollDelta::LineDelta(0.0, 1.0),
        }
    ));
}
