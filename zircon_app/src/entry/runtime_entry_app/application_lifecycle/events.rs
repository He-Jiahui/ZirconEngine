//! Winit 生命周期回调在宿主表面所有权与动态 Runtime 生命周期通知之间的协调层。
//! native 表面只能在可创建回调准入后建立，失效时应先释放宿主资源。

use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_LIFECYCLE_STATE_RESUMED_V1,
    ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1,
};

use super::super::RuntimeEntryApp;
use super::SurfaceReleaseAction;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn handle_application_resumed(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        if self.failure_state.is_recorded() || !self.application_lifecycle.resume() {
            return;
        }
        let event = ZrRuntimeEventV1::lifecycle(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            self.viewport,
            ZR_RUNTIME_LIFECYCLE_STATE_RESUMED_V1,
        );
        self.dispatch_runtime_event(event_loop, event);
    }

    /// 只在 Winit 的表面准入回调执行；成功创建后先确认所有权，再提交初始输入探针。
    pub(in crate::entry::runtime_entry_app) fn handle_surface_availability(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        // Winit admits native surface creation only from can_create_surfaces.
        if self.failure_state.is_recorded()
            || !self.application_lifecycle.surface_creation_requested()
        {
            return;
        }
        if self.create_primary_window_surface(event_loop) {
            self.application_lifecycle.confirm_surface_created();
            if self.submit_mvp_input_probe_if_requested(event_loop) {
                self.request_runtime_frame();
            }
        }
    }

    /// 挂起时先处理宿主表面释放，再通知动态会话；任一失败均请求退出事件循环。
    pub(in crate::entry::runtime_entry_app) fn handle_application_suspended(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        let Some(surface_release) = self.application_lifecycle.suspend() else {
            return;
        };
        let teardown_failed = !self.finish_surface_release(surface_release);
        let event = ZrRuntimeEventV1::lifecycle(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            self.viewport,
            ZR_RUNTIME_LIFECYCLE_STATE_SUSPENDED_V1,
        );
        let event_dispatched =
            !self.failure_state.is_recorded() && self.dispatch_runtime_event(event_loop, event);
        if teardown_failed || !event_dispatched {
            event_loop.exit();
        }
    }

    pub(in crate::entry::runtime_entry_app) fn handle_surface_destruction(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        let surface_release = self.application_lifecycle.destroy_surfaces();
        if !self.finish_surface_release(surface_release) {
            event_loop.exit();
        }
    }

    // TODO: [CR-APP-ENTRY-0004] 确认未使用的 exit/Exiting 路径是否仍需保留；锁定的 Winit 0.31.0-beta.2 没有退出 hook，当前 Drop 已尝试释放主窗口；下一步核对状态机设计去留与 run_app 返回后的所有权。
    pub(in crate::entry::runtime_entry_app) fn handle_application_exit(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        let Some(surface_release) = self.application_lifecycle.exit() else {
            return;
        };
        if !self.finish_surface_release(surface_release) {
            event_loop.exit();
        }
    }

    pub(in crate::entry::runtime_entry_app) fn finish_surface_release(
        &mut self,
        surface_release: SurfaceReleaseAction,
    ) -> bool {
        !surface_release.releases_surface() || self.teardown_primary_window()
    }
}
