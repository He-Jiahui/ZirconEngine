use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use zircon_runtime::diagnostic_log::write_warn;
use zircon_runtime_interface::ZrRuntimeViewportSizeV1;

use super::super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn resize_surface_presenter(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        size: PhysicalSize<u32>,
    ) {
        let viewport_size = ZrRuntimeViewportSizeV1::new(size.width.max(1), size.height.max(1));
        if let Err(error) = self.resize_viewport(viewport_size) {
            self.report_fatal_failure(
                "runtime_surface_present",
                format!(
                    "viewport={:?} size={}x{}",
                    self.viewport, viewport_size.width, viewport_size.height
                ),
                format!("runtime viewport resize failed: {error}"),
                "verify the runtime device and surface state, then restart zircon_runtime",
            );
            event_loop.exit();
            return;
        }
        if self.surface_present_enabled && !self.surface_present_failed {
            match self.bind_current_window_surface() {
                Ok(true) => self.enable_surface_present(),
                Ok(false) => {
                    write_warn(
                        "runtime_surface_present",
                        "runtime_rebind_surface_returned_false",
                    );
                    self.fail_surface_present();
                }
                Err(error) => {
                    write_warn(
                        "runtime_surface_present",
                        format!("runtime_rebind_surface_failed error={error}"),
                    );
                    self.fail_surface_present();
                }
            }
        }
        if let Some(presenter) = self.presenter.as_mut() {
            if let Err(error) = presenter.resize(viewport_size) {
                self.report_fatal_failure(
                    "runtime_surface_present",
                    format!(
                        "fallback_presenter size={}x{}",
                        viewport_size.width, viewport_size.height
                    ),
                    format!("fallback presenter resize failed: {error}"),
                    "verify the graphics adapter and window surface, then restart zircon_runtime",
                );
                event_loop.exit();
            }
        }
    }
}
