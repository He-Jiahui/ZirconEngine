use std::sync::Arc;

use crate::ui::retained_host::primitives::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};
use zircon_runtime::diagnostic_log::write_error;

pub(super) fn create_native_window_or_exit(
    event_loop: &dyn ActiveEventLoop,
    size: PhysicalSize,
) -> Option<Arc<dyn Window>> {
    let window_attributes = WindowAttributes::default()
        .with_title("Zircon Editor")
        .with_surface_size(winit::dpi::LogicalSize::new(
            size.width as f64,
            size.height as f64,
        ));
    match event_loop.create_window(window_attributes) {
        Ok(window) => Some(Arc::from(window)),
        Err(_) => {
            write_error("editor_host_window", "failed to create native window");
            event_loop.exit();
            None
        }
    }
}
