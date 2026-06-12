use winit::event_loop::ActiveEventLoop;
use zircon_runtime::diagnostic_log::write_error;

use super::super::RuntimeEntryApp;
use super::routing::apply_runtime_host_request;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn apply_runtime_host_requests(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) -> bool {
        let requests = match self.session.drain_host_requests() {
            Ok(requests) => requests,
            Err(error) => {
                write_error(
                    "runtime_host_request",
                    format!("runtime_host_request_drain_failed error={error}"),
                );
                event_loop.exit();
                return false;
            }
        };
        for request in requests {
            apply_runtime_host_request(self, request);
        }
        true
    }
}
