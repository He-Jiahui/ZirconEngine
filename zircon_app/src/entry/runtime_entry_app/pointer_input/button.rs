use winit::dpi::PhysicalPosition;
use winit::event::{ButtonSource, ElementState};
use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::{ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

use super::super::{
    converters::{button_state, mouse_button, touch_button_phase},
    RuntimeEntryApp,
};

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn handle_pointer_button(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        state: ElementState,
        button: ButtonSource,
        position: PhysicalPosition<f64>,
    ) {
        if let Some((pointer_id, phase)) = touch_button_phase(&button, state) {
            let event = ZrRuntimeEventV1::touch(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                self.viewport,
                pointer_id,
                phase,
                position.x as f32,
                position.y as f32,
            );
            if self.session.handle_event(event).is_err() {
                event_loop.exit();
            }
        } else if let (Some(button), Some(state)) = (mouse_button(button), button_state(state)) {
            let event = ZrRuntimeEventV1::mouse_button(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                self.viewport,
                button,
                state,
                position.x as f32,
                position.y as f32,
            );
            if self.session.handle_event(event).is_err() {
                event_loop.exit();
            }
        }
    }
}
