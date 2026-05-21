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
        if self.resize_viewport(viewport_size).is_err() {
            event_loop.exit();
            return;
        }
        if self.surface_present_enabled && !self.surface_present_failed {
            match self.bind_current_window_surface() {
                Ok(true) => self.enable_surface_present(),
                Ok(false) => self.fail_surface_present(),
                Err(_) => self.fail_surface_present(),
            }
        }
        if let Some(presenter) = self.presenter.as_mut() {
            if presenter.resize(viewport_size).is_err() {
                event_loop.exit();
            }
        }
    }
}
