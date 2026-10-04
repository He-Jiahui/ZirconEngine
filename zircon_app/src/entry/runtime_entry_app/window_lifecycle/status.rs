//! 主窗口销毁和状态观测的 Runtime 通知。
//! 销毁先清理表面，产品失败时停止派发 Runtime 事件；遮挡先刷新宿主帧策略再通知会话。

use winit::dpi::PhysicalPosition;
use winit::event_loop::ActiveEventLoop;
use winit::window::Theme;
use zircon_runtime_interface::{ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

use super::super::{converters::window_theme, RuntimeEntryApp};

impl RuntimeEntryApp {
    /// 销毁通知先尝试表面清理，再派发 Runtime 事件；任一步失败或窗口策略要求时退出。
    pub(in crate::entry::runtime_entry_app) fn handle_window_destroyed(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        let surface_release = self.application_lifecycle.destroy_surfaces();
        let teardown_failed = !self.finish_surface_release(surface_release);
        let event =
            ZrRuntimeEventV1::window_destroyed(ZIRCON_RUNTIME_ABI_VERSION_V1, self.viewport);
        let event_dispatched =
            !self.failure_state.is_recorded() && self.dispatch_runtime_event(event_loop, event);
        if teardown_failed
            || !event_dispatched
            || self
                .window_lifecycle_policy
                .should_exit_after_primary_close()
        {
            event_loop.exit();
        }
    }

    pub(in crate::entry::runtime_entry_app) fn handle_window_moved(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        position: PhysicalPosition<i32>,
    ) {
        let event = ZrRuntimeEventV1::window_moved(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            self.viewport,
            position.x,
            position.y,
        );
        self.dispatch_runtime_event(event_loop, event);
    }

    /// 同步宿主帧节流与 Runtime 遮挡输入；变化时请求一次状态帧。
    pub(in crate::entry::runtime_entry_app) fn handle_window_occluded(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        occluded: bool,
    ) {
        if self.frame_cadence.set_window_occluded(occluded) {
            self.request_runtime_frame();
        }
        let event = ZrRuntimeEventV1::window_occluded(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            self.viewport,
            occluded,
        );
        self.dispatch_runtime_event(event_loop, event);
    }

    pub(in crate::entry::runtime_entry_app) fn handle_window_theme_changed(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        theme: Theme,
    ) {
        let event = ZrRuntimeEventV1::window_theme_changed(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            self.viewport,
            window_theme(theme),
        );
        self.dispatch_runtime_event(event_loop, event);
    }
}
