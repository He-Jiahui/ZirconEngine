use winit::event::DeviceEvent;
use winit::event_loop::ActiveEventLoop;

use super::super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn handle_device_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        event: DeviceEvent,
    ) {
        self.handle_pointer_device_event(event_loop, event);
    }
}
