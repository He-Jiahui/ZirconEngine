use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::{ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

use super::super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn handle_window_scale_factor_changed(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        scale_factor: f64,
    ) {
        let scale_factor = scale_factor as f32;
        let backend_event = ZrRuntimeEventV1::window_backend_scale_factor_changed(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            self.viewport,
            scale_factor,
        );
        if self.session.handle_event(backend_event).is_err() {
            event_loop.exit();
            return;
        }

        let logical_event = ZrRuntimeEventV1::window_scale_factor_changed(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            self.viewport,
            scale_factor,
        );
        if self.session.handle_event(logical_event).is_err() {
            event_loop.exit();
        }
    }
}
