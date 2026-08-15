use winit::event::DeviceEvent;
use winit::event_loop::ActiveEventLoop;

use super::super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn handle_device_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        event: DeviceEvent,
    ) {
        if !device_event_requests_runtime_frame(&event) {
            return;
        }
        self.request_runtime_frame();
        self.handle_pointer_device_event(event_loop, event);
    }
}

fn device_event_requests_runtime_frame(event: &DeviceEvent) -> bool {
    matches!(event, DeviceEvent::PointerMotion { .. })
}

#[cfg(test)]
mod tests {
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
}
