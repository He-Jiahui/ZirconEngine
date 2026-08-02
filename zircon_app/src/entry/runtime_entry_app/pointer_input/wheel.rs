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
        let event = if let Some(position) = self.last_pointer_position {
            ZrRuntimeEventV1::mouse_wheel_delta_at(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                self.viewport,
                unit,
                position.x as f32,
                position.y as f32,
                x,
                y,
            )
        } else {
            ZrRuntimeEventV1::mouse_wheel_delta(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                self.viewport,
                unit,
                x,
                y,
            )
        };
        self.dispatch_runtime_event(event_loop, event);
    }
}
