//! 同一系统 DPI 变化在 Runtime 后端状态与逻辑窗口状态的有序通知。
//! 先后端再逻辑；第一条失败就停止传播。

use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::{ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

use super::super::RuntimeEntryApp;

impl RuntimeEntryApp {
    /// 同一 DPI 变化先通知 Runtime 后端，再通知逻辑窗口消费者。
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
        if !self.dispatch_runtime_event(event_loop, backend_event) {
            return;
        }

        let logical_event = ZrRuntimeEventV1::window_scale_factor_changed(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            self.viewport,
            scale_factor,
        );
        self.dispatch_runtime_event(event_loop, logical_event);
    }
}
