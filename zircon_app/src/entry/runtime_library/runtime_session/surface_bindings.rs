//! RuntimeSession 对宿主共享视口绑定状态机的窄封装。
//! 需要执行 ABI 的迁移先预留 token，再按结果提交或回滚；无已发布绑定的释放不调用 ABI。
//! 缺少对应能力时会话可短路返回；同视口已有迁移时当前请求返回错误，由调用方决定何时重试。

use zircon_runtime_host::viewport_surface::{
    ViewportSurfaceBindingOperation, ViewportSurfaceOperationInFlight,
    ViewportSurfaceReleaseOperation,
};
use zircon_runtime_interface::ZrRuntimeViewportHandle;

use super::super::RuntimeLibraryError;
use super::RuntimeSession;

impl RuntimeSession {
    /// 需要执行 FFI 绑定时预留迁移 token；同视口已有迁移则拒绝本次请求，不在此等待。
    pub(super) fn begin_viewport_surface_binding(
        &self,
        viewport: ZrRuntimeViewportHandle,
    ) -> Result<ViewportSurfaceBindingOperation<'_>, RuntimeLibraryError> {
        self.viewport_surface_bindings
            .begin_binding(viewport)
            .map_err(viewport_surface_operation_in_flight_error)
    }

    pub(super) fn finish_viewport_surface_binding(
        &self,
        operation: ViewportSurfaceBindingOperation<'_>,
        succeeded: bool,
    ) {
        operation.finish(succeeded);
    }

    pub(super) fn begin_viewport_surface_release(
        &self,
        viewport: ZrRuntimeViewportHandle,
    ) -> Result<Option<ViewportSurfaceReleaseOperation<'_>>, RuntimeLibraryError> {
        self.viewport_surface_bindings
            .begin_release(viewport)
            .map_err(viewport_surface_operation_in_flight_error)
    }

    pub(super) fn finish_viewport_surface_release(
        &self,
        operation: ViewportSurfaceReleaseOperation<'_>,
        succeeded: bool,
    ) {
        operation.finish(succeeded);
    }

    /// 供会话析构按宿主记录的已发布绑定逐一尝试释放。
    pub(super) fn bound_viewport_surfaces(&self) -> Vec<ZrRuntimeViewportHandle> {
        self.viewport_surface_bindings.bound_viewports()
    }
}

fn viewport_surface_operation_in_flight_error(
    operation: ViewportSurfaceOperationInFlight,
) -> RuntimeLibraryError {
    RuntimeLibraryError::new(format!(
        "viewport surface binding transition is already in flight for viewport {}",
        operation.viewport().raw()
    ))
}
