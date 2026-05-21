use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::{ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

use super::super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn handle_file_drag_cancelled(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        let event =
            ZrRuntimeEventV1::file_drag_cancelled(ZIRCON_RUNTIME_ABI_VERSION_V1, self.viewport);
        if self.session.handle_event(event).is_err() {
            event_loop.exit();
        }
    }
}
