//! 动态 Runtime 在帧末发往宿主的请求批次入口。
//! 排空失败终止产品；单项操作的后续错误策略由路由层决定。

use winit::event_loop::ActiveEventLoop;

use super::super::RuntimeEntryApp;
use super::routing::apply_runtime_host_request;

impl RuntimeEntryApp {
    /// tick 后、重绘前消费完整请求批次；排空失败记录为产品终止原因。
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
        #[cfg(feature = "gamepad-gilrs")]
        if !requests.is_empty() {
            super::super::gamepad::clear_finished_rumble_effects(
                self.gamepad_rumble_effects.as_mut(),
            );
        }
        for request in requests {
            apply_runtime_host_request(self, event_loop, request);
        }
        true
    }
}
