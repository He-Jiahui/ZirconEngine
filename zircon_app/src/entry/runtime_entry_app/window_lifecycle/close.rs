use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::{ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

use super::super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn handle_window_close_requested(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        let event =
            ZrRuntimeEventV1::window_close_requested(ZIRCON_RUNTIME_ABI_VERSION_V1, self.viewport);
        if self.session.handle_event(event).is_err() {
            event_loop.exit();
            return;
        }
        if self.window_lifecycle_policy.should_close_on_request() {
            self.close_primary_window_after_request();
            if self
                .window_lifecycle_policy
                .should_exit_after_primary_close()
            {
                event_loop.exit();
            }
        }
    }
}
