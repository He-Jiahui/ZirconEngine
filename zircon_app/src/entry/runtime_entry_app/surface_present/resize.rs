use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::ZrRuntimeViewportSizeV1;

use super::super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn resize_surface_presenter(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        size: PhysicalSize<u32>,
    ) {
        let viewport_size = ZrRuntimeViewportSizeV1::new(size.width.max(1), size.height.max(1));
        if viewport_size == self.viewport_size {
            return;
        }
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
        if self.surface_present_enabled {
            match self.bind_current_window_surface() {
                Ok(true) => self.enable_surface_present(),
                Ok(false) => {
                    self.report_fatal_failure(
                        "runtime_surface_present",
                        format!(
                            "viewport={:?} size={}x{}",
                            self.viewport, viewport_size.width, viewport_size.height
                        ),
                        "native surface rebind returned unavailable after a successful bind",
                        "verify the runtime surface contract and restart zircon_runtime",
                    );
                    event_loop.exit();
                    return;
                }
                Err(error) => {
                    self.report_fatal_failure(
                        "runtime_surface_present",
                        format!(
                            "viewport={:?} size={}x{}",
                            self.viewport, viewport_size.width, viewport_size.height
                        ),
                        format!("native surface rebind failed: {error}"),
                        "verify the graphics adapter and window surface, then restart zircon_runtime",
                    );
                    event_loop.exit();
                    return;
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

pub(in crate::entry::runtime_entry_app) fn surface_resize_changes_viewport(
    viewport_size: ZrRuntimeViewportSizeV1,
    size: PhysicalSize<u32>,
) -> bool {
    viewport_size != ZrRuntimeViewportSizeV1::new(size.width.max(1), size.height.max(1))
}

#[cfg(test)]
mod tests {
    use winit::dpi::PhysicalSize;
    use zircon_runtime_interface::ZrRuntimeViewportSizeV1;

    use super::surface_resize_changes_viewport;

    #[test]
    fn duplicate_surface_resize_is_a_no_op_after_minimum_size_normalization() {
        let current = ZrRuntimeViewportSizeV1::new(1, 720);

        assert!(!surface_resize_changes_viewport(
            current,
            PhysicalSize::new(0, 720),
        ));
        assert!(surface_resize_changes_viewport(
            current,
            PhysicalSize::new(2, 720),
        ));
    }
}
