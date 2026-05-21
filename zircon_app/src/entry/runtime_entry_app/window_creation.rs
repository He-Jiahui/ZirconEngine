use std::sync::Arc;

use winit::event_loop::ActiveEventLoop;
use winit::window::Window;
use zircon_runtime_interface::ZrRuntimeViewportSizeV1;

use super::{window_attributes::runtime_window_attributes, RuntimeEntryApp};

impl RuntimeEntryApp {
    pub(super) fn create_primary_window_surface(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        // Minimal/headless runtime sessions intentionally run without a concrete primary window.
        if self.window_descriptor.primary_window.is_none() {
            return;
        }

        let window_attributes = runtime_window_attributes(&self.window_descriptor, event_loop);
        let window: Arc<dyn Window> = match event_loop.create_window(window_attributes) {
            Ok(window) => Arc::from(window),
            Err(_) => {
                event_loop.exit();
                return;
            }
        };
        let size = window.surface_size();
        let viewport_size = ZrRuntimeViewportSizeV1::new(size.width.max(1), size.height.max(1));
        self.window = Some(window.clone());
        self.viewport_size = viewport_size;
        if self.resize_viewport(viewport_size).is_err() {
            event_loop.exit();
            return;
        }
        match self.bind_window_surface(window.as_ref()) {
            Ok(true) => self.enable_surface_present(),
            Ok(false) => self.fallback_surface_present(),
            Err(_) => {
                self.fail_surface_present();
            }
        }
        if !self.surface_present_enabled && !self.ensure_fallback_presenter(event_loop) {
            return;
        }
    }
}
