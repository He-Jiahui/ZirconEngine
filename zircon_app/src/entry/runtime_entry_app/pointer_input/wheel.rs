use winit::event::MouseScrollDelta;
use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::{ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

use super::super::{converters::mouse_wheel_delta, RuntimeEntryApp};

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn handle_mouse_wheel(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        delta: MouseScrollDelta,
    ) {
        let (unit, x, y) = mouse_wheel_delta(delta);
        let event = ZrRuntimeEventV1::mouse_wheel_delta(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            self.viewport,
            unit,
            x,
            y,
        );
        if self.session.handle_event(event).is_err() {
            event_loop.exit();
        }
    }
}
