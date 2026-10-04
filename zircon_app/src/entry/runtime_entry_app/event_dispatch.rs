//! 宿主事件转交动态 Runtime 的共同失败边界。
//! 多步派发可用 false 停止同一回调的后续处理；单项回调依靠失败标志与事件循环退出收尾。

use std::fmt::Display;

use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::{ZrRuntimeEventV1, ZrRuntimeViewportHandle};

use super::{failure::RuntimeEntryAppFailure, RuntimeEntryApp};

impl RuntimeEntryApp {
    /// 输入和窗口回调共用；失败会记录产品终止原因并要求 Winit 退出。
    pub(super) fn dispatch_runtime_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        event: ZrRuntimeEventV1,
    ) -> bool {
        let event_kind = event.kind;
        match self.session.handle_event(event) {
            Ok(()) => true,
            Err(error) => {
                let failure = runtime_event_dispatch_failure(event_kind, self.viewport, error);
                zircon_runtime::diagnostic_log::write_error(
                    "runtime_event_dispatch",
                    failure.to_string(),
                );
                self.failure_state.record(failure);
                event_loop.exit();
                false
            }
        }
    }
}

fn runtime_event_dispatch_failure(
    event_kind: u32,
    viewport: ZrRuntimeViewportHandle,
    error: impl Display,
) -> RuntimeEntryAppFailure {
    RuntimeEntryAppFailure::new(
        "runtime_event_dispatch",
        format!("event_kind={event_kind} viewport={viewport:?}"),
        format!("runtime event dispatch failed: {error}"),
        "verify the runtime library ABI and event handler, then restart zircon_runtime",
    )
}

#[cfg(test)]
#[path = "tests/event_dispatch.rs"]
mod tests;
