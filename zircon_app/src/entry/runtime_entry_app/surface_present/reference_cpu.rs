//! 显式诊断模式的窗口 CPU presenter 懒创建边界。
//! 非显式请求和创建失败都形成产品错误，不能隐式回退。

use winit::event_loop::ActiveEventLoop;
use zircon_runtime::diagnostic_log::write_log;

use super::super::RuntimeEntryApp;
use crate::reference_cpu_presenter::ReferenceCpuPresenter;

impl RuntimeEntryApp {
    /// 只为显式降级诊断创建并保留 presenter；缺少 opt-in 属于产品错误。
    pub(in crate::entry::runtime_entry_app) fn ensure_reference_cpu_presenter(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) -> bool {
        if !self.reference_cpu_presenter_enabled {
            self.report_fatal_failure(
                "runtime_surface_present",
                "reference_cpu_presenter",
                "reference CPU presenter was requested without --reference-cpu-presenter",
                "run with a qualified native surface backend, or explicitly pass --reference-cpu-presenter for degraded diagnostics",
            );
            event_loop.exit();
            return false;
        }
        if self.presenter.is_some() {
            return true;
        }
        let Some(window) = self.window.as_ref() else {
            return false;
        };
        match ReferenceCpuPresenter::new(window.clone()) {
            Ok(presenter) => {
                self.presenter = Some(presenter);
                write_log(
                    "runtime_surface_present",
                    format!(
                        "runtime_reference_cpu_presenter_created capability=degraded size={}x{}",
                        self.viewport_size.width, self.viewport_size.height
                    ),
                );
                true
            }
            Err(error) => {
                self.report_fatal_failure(
                    "runtime_surface_present",
                    format!(
                        "reference_cpu_presenter size={}x{}",
                        self.viewport_size.width, self.viewport_size.height
                    ),
                    format!("reference CPU presenter creation failed: {error}"),
                    "verify the graphics adapter and window surface, then restart zircon_runtime",
                );
                event_loop.exit();
                false
            }
        }
    }
}
