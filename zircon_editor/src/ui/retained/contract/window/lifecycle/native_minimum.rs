use winit::dpi::{PhysicalSize, Size};
use winit::window::Window;

use crate::ui::retained_host::host_contract::window::UiHostWindow;
use crate::ui::workbench::autolayout::{
    window_min_height_limit_for_height, window_min_width_limit_for_physical_width,
};

use super::super::UiHostWindowEventLoop;

const DEFAULT_SCALE_FACTOR: f32 = 1.0;

pub(super) fn current_native_minimum_surface_size(host: &UiHostWindow) -> PhysicalSize<u32> {
    let handle = host.window();
    let state = handle.state.borrow();
    let shell = &state.host_presentation.host_shell;
    let scale = if state.window_scale_factor.is_finite() && state.window_scale_factor > 0.0 {
        state.window_scale_factor
    } else {
        DEFAULT_SCALE_FACTOR
    };
    let fallback_width =
        window_min_width_limit_for_physical_width(state.window_size.width as f32, scale);
    let fallback_height =
        window_min_height_limit_for_height(state.window_size.height as f32 / scale);
    PhysicalSize::new(
        published_or_fallback_physical_extent(shell.shell_min_width_px, fallback_width, scale),
        published_or_fallback_physical_extent(shell.shell_min_height_px, fallback_height, scale),
    )
}

fn published_or_fallback_physical_extent(
    published_physical: f32,
    fallback_logical: f32,
    scale_factor: f32,
) -> u32 {
    if published_physical.is_finite() && published_physical > 0.0 {
        physical_extent(published_physical)
    } else {
        logical_to_physical(fallback_logical, normalized_scale(scale_factor))
    }
}

#[cfg(test)]
fn physical_minimum_size(
    logical_width: f32,
    logical_height: f32,
    scale_factor: f32,
) -> PhysicalSize<u32> {
    let scale = normalized_scale(scale_factor);
    PhysicalSize::new(
        logical_to_physical(logical_width, scale),
        logical_to_physical(logical_height, scale),
    )
}

fn normalized_scale(scale_factor: f32) -> f64 {
    if scale_factor.is_finite() && scale_factor > 0.0 {
        f64::from(scale_factor)
    } else {
        f64::from(DEFAULT_SCALE_FACTOR)
    }
}

fn physical_extent(physical: f32) -> u32 {
    let physical = if physical.is_finite() {
        f64::from(physical.max(0.0))
    } else {
        0.0
    };
    physical.ceil().min(f64::from(u32::MAX)) as u32
}

fn logical_to_physical(logical: f32, scale: f64) -> u32 {
    let logical = if logical.is_finite() {
        f64::from(logical.max(0.0))
    } else {
        0.0
    };
    (logical * scale).ceil().min(f64::from(u32::MAX)) as u32
}

impl UiHostWindowEventLoop {
    pub(in crate::ui::retained_host::host_contract::window::event_loop) fn sync_native_window_minimum_from_presentation(
        &mut self,
    ) {
        let Some(window) = self.window.clone() else {
            return;
        };
        self.sync_native_window_minimum(window.as_ref());
    }

    pub(in crate::ui::retained_host::host_contract::window::event_loop) fn sync_native_window_minimum(
        &mut self,
        window: &dyn Window,
    ) {
        let minimum = current_native_minimum_surface_size(&self.host);
        let key = (minimum.width, minimum.height);
        if self.native_minimum_surface_size == Some(key) {
            return;
        }
        window.set_min_surface_size(Some(Size::Physical(minimum)));
        self.native_minimum_surface_size = Some(key);
    }
}

#[cfg(test)]
#[path = "tests/native_minimum.rs"]
mod tests;
