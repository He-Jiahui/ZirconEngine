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
        if !self.dispatch_runtime_event(event_loop, event) {
            return;
        }
        if self.window_lifecycle_policy.should_close_on_request() {
            let surface_release = self.application_lifecycle.destroy_surfaces();
            let teardown_failed = !self.finish_surface_release(surface_release);
            if teardown_failed
                || self
                    .window_lifecycle_policy
                    .should_exit_after_primary_close()
            {
                event_loop.exit();
            }
        }
    }
}
