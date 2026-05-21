use winit::event::DeviceEvent;
use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::{ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

use super::super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn handle_pointer_device_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        event: DeviceEvent,
    ) {
        let DeviceEvent::PointerMotion {
            delta: (delta_x, delta_y),
        } = event
        else {
            return;
        };
        let event = ZrRuntimeEventV1::mouse_motion(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            self.viewport,
            delta_x as f32,
            delta_y as f32,
        );
        if self.session.handle_event(event).is_err() {
            event_loop.exit();
        }
    }
}
