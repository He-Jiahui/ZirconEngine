use winit::dpi::PhysicalPosition;
use winit::event::PointerKind;
use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1,
};

use super::super::{converters::pointer_kind_touch_id, RuntimeEntryApp};

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn handle_pointer_entered(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        let event = ZrRuntimeEventV1::cursor_entered(ZIRCON_RUNTIME_ABI_VERSION_V1, self.viewport);
        if self.session.handle_event(event).is_err() {
            event_loop.exit();
        }
    }

    pub(in crate::entry::runtime_entry_app) fn handle_pointer_left(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        position: Option<PhysicalPosition<f64>>,
        kind: PointerKind,
    ) {
        let event = ZrRuntimeEventV1::cursor_left(ZIRCON_RUNTIME_ABI_VERSION_V1, self.viewport);
        if self.session.handle_event(event).is_err() {
            event_loop.exit();
        }
        if let Some(pointer_id) = pointer_kind_touch_id(kind) {
            let event = ZrRuntimeEventV1::touch(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                self.viewport,
                pointer_id,
                ZR_RUNTIME_TOUCH_PHASE_CANCELLED_V1,
                position
                    .map(|position| position.x as f32)
                    .unwrap_or_default(),
                position
                    .map(|position| position.y as f32)
                    .unwrap_or_default(),
            );
            if self.session.handle_event(event).is_err() {
                event_loop.exit();
            }
        }
    }
}
