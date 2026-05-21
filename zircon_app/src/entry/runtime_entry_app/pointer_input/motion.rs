use winit::dpi::PhysicalPosition;
use winit::event::PointerSource;
use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_TOUCH_PHASE_MOVED_V1,
};

use super::super::{converters::pointer_source_touch_id, RuntimeEntryApp};

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn handle_pointer_moved(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        position: PhysicalPosition<f64>,
        source: PointerSource,
    ) {
        let event = if let Some(pointer_id) = pointer_source_touch_id(&source) {
            ZrRuntimeEventV1::touch(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                self.viewport,
                pointer_id,
                ZR_RUNTIME_TOUCH_PHASE_MOVED_V1,
                position.x as f32,
                position.y as f32,
            )
        } else {
            ZrRuntimeEventV1::pointer_moved(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                self.viewport,
                position.x as f32,
                position.y as f32,
            )
        };
        if self.session.handle_event(event).is_err() {
            event_loop.exit();
        }
    }
}
