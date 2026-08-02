use std::sync::Arc;

use crate::ui::retained_host::primitives::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

use super::super::super::UiHostWindow;

pub(super) fn create_native_window_or_exit(
    event_loop: &dyn ActiveEventLoop,
    host: &UiHostWindow,
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
        Err(error) => {
            host.report_fatal_failure(
                "editor_host_window",
                format!("native_window size={}x{}", size.width, size.height),
                format!("native window creation failed: {error}"),
                "verify the desktop session can create windows and retry zircon_editor",
            );
            event_loop.exit();
            None
        }
    }
}
