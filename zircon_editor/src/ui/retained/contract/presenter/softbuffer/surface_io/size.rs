use std::num::NonZeroU32;
use std::sync::Arc;

use softbuffer::Surface;
use winit::window::Window;

pub(in crate::ui::retained_host::host_contract) fn current_window_size(
    window: &dyn Window,
) -> (u32, u32) {
    let size = window.surface_size();
    clamp_size((size.width, size.height))
}

pub(in crate::ui::retained_host::host_contract) fn resize_surface(
    surface: &mut Surface<Arc<dyn Window>, Arc<dyn Window>>,
    size: (u32, u32),
) -> Result<(), softbuffer::SoftBufferError> {
    surface.resize(non_zero(size.0), non_zero(size.1))
}

pub(in crate::ui::retained_host::host_contract) fn clamp_size(size: (u32, u32)) -> (u32, u32) {
    (size.0.max(1), size.1.max(1))
}

fn non_zero(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value.max(1)).expect("value is clamped to non-zero")
}
