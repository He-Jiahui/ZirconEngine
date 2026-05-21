use winit::event_loop::ActiveEventLoop;

use super::super::RuntimeEntryApp;
use super::routing::apply_runtime_host_request;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn apply_runtime_host_requests(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) -> bool {
        let requests = match self.session.drain_host_requests() {
            Ok(requests) => requests,
            Err(_) => {
                event_loop.exit();
                return false;
            }
        };
        let Some(window) = self.window.as_ref() else {
            return true;
        };
        for request in requests {
            apply_runtime_host_request(window.as_ref(), request);
        }
        true
    }
}
