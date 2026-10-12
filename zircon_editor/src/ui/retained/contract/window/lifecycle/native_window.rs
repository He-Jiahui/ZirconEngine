use std::sync::Arc;

use crate::ui::retained_host::primitives::PhysicalSize;
use winit::dpi::{PhysicalSize as WinitPhysicalSize, Size};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

use super::super::super::UiHostWindow;
use crate::ui::retained_host::host_contract::profiling_artifacts::profile_capture_enabled;

const PROFILE_INITIAL_CLIENT_WIDTH_ENV: &str = "ZIRCON_PROFILE_INITIAL_CLIENT_WIDTH";
const PROFILE_INITIAL_CLIENT_HEIGHT_ENV: &str = "ZIRCON_PROFILE_INITIAL_CLIENT_HEIGHT";

pub(super) fn create_native_window_or_exit(
    event_loop: &dyn ActiveEventLoop,
    host: &UiHostWindow,
    size: PhysicalSize,
    minimum_size: WinitPhysicalSize<u32>,
) -> Option<Arc<dyn Window>> {
    let window_attributes = native_window_attributes(&size, minimum_size);
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

fn native_window_attributes(
    size: &PhysicalSize,
    minimum_size: WinitPhysicalSize<u32>,
) -> WindowAttributes {
    let size = profile_initial_client_size().unwrap_or_else(|| size.clone());
    native_window_attributes_for_size(&size, minimum_size)
}

fn native_window_attributes_for_size(
    size: &PhysicalSize,
    minimum_size: WinitPhysicalSize<u32>,
) -> WindowAttributes {
    WindowAttributes::default()
        .with_title("Zircon Editor")
        .with_surface_size(Size::Physical(WinitPhysicalSize::new(
            size.width,
            size.height,
        )))
        .with_min_surface_size(Size::Physical(minimum_size))
}

fn profile_initial_client_size() -> Option<PhysicalSize> {
    profile_capture_enabled().then(|| {
        parse_profile_initial_client_size(
            std::env::var(PROFILE_INITIAL_CLIENT_WIDTH_ENV)
                .ok()
                .as_deref(),
            std::env::var(PROFILE_INITIAL_CLIENT_HEIGHT_ENV)
                .ok()
                .as_deref(),
        )
    })?
}

fn parse_profile_initial_client_size(
    width: Option<&str>,
    height: Option<&str>,
) -> Option<PhysicalSize> {
    let width = width?.parse::<u32>().ok().filter(|value| *value > 0)?;
    let height = height?.parse::<u32>().ok().filter(|value| *value > 0)?;
    Some(PhysicalSize::new(width, height))
}

#[cfg(test)]
#[path = "tests/native_window.rs"]
mod tests;
