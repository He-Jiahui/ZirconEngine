//! Winit 可创建表面回调中的主窗口与动态 Runtime surface 建立。
//! 描述符可明确无主窗口；窗口 Arc 必须比已绑定的原生表面活得更久。

use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;
use winit::window::Window;
use zircon_runtime::diagnostic_log::write_log;
use zircon_runtime_interface::ZrRuntimeViewportSizeV1;

use super::{window_attributes::runtime_window_attributes, RuntimeEntryApp};

impl RuntimeEntryApp {
    /// 由 Winit 表面可用回调调用；无主窗口配置保持 headless，原生绑定失败形成产品诊断。
    pub(super) fn create_primary_window_surface(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) -> bool {
        if self.failure_state.is_recorded() {
            return false;
        }
        if self.window.is_some() {
            return true;
        }
        // Minimal/headless runtime sessions intentionally run without a concrete primary window.
        if self.window_descriptor.primary_window.is_none() {
            return true;
        }

        if let Err(error) = self.validate_native_ime_v2_request() {
            self.report_fatal_failure(
                "runtime_ime",
                "native_app_session_v2",
                error,
                "disable native IME V2 or provide a registered IMM/TSF callback adapter",
            );
            event_loop.exit();
            return false;
        }

        let window_attributes = runtime_window_attributes(&self.window_descriptor, event_loop);
        let window: Arc<dyn Window> = match event_loop.create_window(window_attributes) {
            Ok(window) => Arc::from(window),
            Err(error) => {
                self.report_fatal_failure(
                    "runtime_window",
                    "primary_window",
                    format!("window creation failed: {error}"),
                    "verify the desktop session can create windows and retry zircon_runtime",
                );
                event_loop.exit();
                return false;
            }
        };
        let size = window.surface_size();
        let viewport_size = ZrRuntimeViewportSizeV1::new(size.width.max(1), size.height.max(1));
        self.window = Some(window.clone());
        self.viewport_size = viewport_size;
        write_log(
            "runtime_window",
            format!(
                "runtime_primary_window_created viewport={:?} size={}x{}",
                self.viewport, viewport_size.width, viewport_size.height
            ),
        );
        if let Err(error) = self.resize_viewport(viewport_size) {
            self.report_fatal_failure(
                "runtime_window",
                format!(
                    "viewport={:?} size={}x{}",
                    self.viewport, viewport_size.width, viewport_size.height
                ),
                format!("viewport resize failed: {error}"),
                "verify runtime device initialization and restart zircon_runtime",
            );
            event_loop.exit();
            return false;
        }
        match self.bind_window_surface(window.as_ref()) {
            Ok(true) => self.enable_surface_present(),
            Ok(false) => {
                write_log(
                    "runtime_surface_present",
                    "runtime_bind_window_surface_degraded_reference_cpu",
                );
                if !self.enable_reference_cpu_presenter() {
                    event_loop.exit();
                    return false;
                }
            }
            Err(error) => {
                self.report_fatal_failure(
                    "runtime_surface_present",
                    format!(
                        "viewport={:?} size={}x{}",
                        self.viewport, viewport_size.width, viewport_size.height
                    ),
                    format!("runtime window surface bind failed: {error}"),
                    "verify the graphics adapter and window surface, then restart zircon_runtime",
                );
                event_loop.exit();
                return false;
            }
        }
        if self.surface_present_enabled {
            true
        } else {
            self.ensure_reference_cpu_presenter(event_loop)
        }
    }
}
