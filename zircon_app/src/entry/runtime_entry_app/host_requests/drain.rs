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
            Err(error) => {
                self.report_fatal_failure(
                    "runtime_host_request",
                    "drain_pending_requests",
                    format!("runtime host request drain failed: {error}"),
                    "verify the runtime library ABI and host-request queue, then restart zircon_runtime",
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
