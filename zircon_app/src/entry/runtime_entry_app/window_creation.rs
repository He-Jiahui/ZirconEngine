use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;
use winit::window::Window;
use zircon_runtime::diagnostic_log::write_log;
use zircon_runtime_interface::ZrRuntimeViewportSizeV1;

use super::{window_attributes::runtime_window_attributes, RuntimeEntryApp};

impl RuntimeEntryApp {
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
                    "runtime_bind_window_surface_unavailable",
                );
                self.fallback_surface_present();
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
            self.ensure_fallback_presenter(event_loop)
        }
    }
}
