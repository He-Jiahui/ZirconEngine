//! 在窗口创建或尺寸变化后，把有效原生窗口 target 交给动态 Runtime。
//! 只有显式选择的 CPU 诊断路径可绕开原生绑定；attempted 状态服务异常清理。

use winit::window::Window;
use zircon_runtime::diagnostic_log::write_log;
use zircon_runtime_interface::{
    ZrRuntimeBindViewportSurfaceRequestV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

use super::super::{
    window_surface::{runtime_native_surface_target, NativeSurfaceTargetUnavailable},
    RuntimeEntryApp,
};
use crate::entry::runtime_library::RuntimeLibraryError;

impl RuntimeEntryApp {
    /// 在 resize 后重新使用当前主窗口；无窗口时保持无表面会话。
    pub(in crate::entry::runtime_entry_app) fn bind_current_window_surface(
        &mut self,
    ) -> Result<bool, RuntimeLibraryError> {
        let Some(window) = self.window.clone() else {
            return Ok(false);
        };
        self.bind_window_surface(window.as_ref())
    }

    /// 要求完整原生呈现能力后绑定目标；显式 CPU 诊断模式返回未绑定。
    pub(in crate::entry::runtime_entry_app) fn bind_window_surface(
        &mut self,
        window: &dyn Window,
    ) -> Result<bool, RuntimeLibraryError> {
        if self.reference_cpu_presenter_enabled {
            write_log(
                "runtime_surface_present",
                "runtime_reference_cpu_presenter_enabled capability=degraded",
            );
            return Ok(false);
        }
        if !self.session.supports_viewport_surface_present() {
            return Err(RuntimeLibraryError::capability_unavailable(
                "runtime did not export viewport surface presentation; use --reference-cpu-presenter only for a degraded diagnostic path",
            ));
        }
        let target =
            runtime_native_surface_target(window).map_err(native_surface_unavailable_error)?;
        self.surface_present_attempted = true;
        self.session
            .bind_viewport_surface(ZrRuntimeBindViewportSurfaceRequestV1::new(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                self.viewport,
                self.viewport_size,
                target,
            ))
    }
}

fn native_surface_unavailable_error(
    unavailable: NativeSurfaceTargetUnavailable,
) -> RuntimeLibraryError {
    RuntimeLibraryError::capability_unavailable(format!(
        "qualified native surface target unavailable: {unavailable}; use --reference-cpu-presenter only for a degraded diagnostic path"
    ))
}
