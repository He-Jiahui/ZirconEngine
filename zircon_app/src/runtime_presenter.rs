use std::num::NonZeroU32;
use std::sync::Arc;

use softbuffer::{Context, Surface};
use winit::window::Window;
use zircon_runtime_interface::ZrRuntimeViewportSizeV1;

use crate::entry::runtime_library::RuntimeFrame;

pub(crate) struct SoftbufferRuntimePresenter {
    _context: Context<Arc<dyn Window>>,
    surface: Surface<Arc<dyn Window>, Arc<dyn Window>>,
    size: ZrRuntimeViewportSizeV1,
}

impl SoftbufferRuntimePresenter {
    pub(crate) fn new(window: Arc<dyn Window>) -> Result<Self, softbuffer::SoftBufferError> {
        let context = Context::new(window.clone())?;
        let mut surface = Surface::new(&context, window.clone())?;
        let size = current_window_size(window.as_ref());
        resize_surface(&mut surface, size)?;
        Ok(Self {
            _context: context,
            surface,
            size,
        })
    }

    pub(crate) fn resize(
        &mut self,
        size: ZrRuntimeViewportSizeV1,
    ) -> Result<(), softbuffer::SoftBufferError> {
        let size = clamp_size(size);
        resize_surface(&mut self.surface, size)?;
        self.size = size;
        Ok(())
    }

    pub(crate) fn present(
        &mut self,
        frame: &RuntimeFrame<'_>,
    ) -> Result<(), softbuffer::SoftBufferError> {
        zircon_runtime::profile_scope!("app", "runtime_presenter", "present");
        let frame_size = ZrRuntimeViewportSizeV1::new(frame.width().max(1), frame.height().max(1));
        if self.size != frame_size {
            self.resize(frame_size)?;
        }

        let window = self.surface.window().clone();
        let mut buffer = self.surface.buffer_mut()?;
        {
            zircon_runtime::profile_scope!("app", "runtime_presenter", "copy_rgba");
            copy_rgba_to_xrgb(&mut buffer, frame.rgba());
        }

        window.pre_present_notify();
        zircon_runtime::profile_scope!("app", "runtime_presenter", "softbuffer_present");
        buffer.present()
    }
}

fn copy_rgba_to_xrgb(surface: &mut [u32], rgba: &[u8]) -> bool {
    let covers_surface = surface
        .len()
        .checked_mul(4)
        .is_some_and(|required_bytes| rgba.len() >= required_bytes);
    if !covers_surface {
        surface.fill(0);
    }
    for (pixel, rgba) in surface.iter_mut().zip(rgba.chunks_exact(4)) {
        let red = rgba[0] as u32;
        let green = rgba[1] as u32;
        let blue = rgba[2] as u32;
        *pixel = (red << 16) | (green << 8) | blue;
    }
    !covers_surface
}

fn current_window_size(window: &dyn Window) -> ZrRuntimeViewportSizeV1 {
    let size = window.surface_size();
    ZrRuntimeViewportSizeV1::new(size.width.max(1), size.height.max(1))
}

fn resize_surface(
    surface: &mut Surface<Arc<dyn Window>, Arc<dyn Window>>,
    size: ZrRuntimeViewportSizeV1,
) -> Result<(), softbuffer::SoftBufferError> {
    surface.resize(non_zero(size.width), non_zero(size.height))
}

fn clamp_size(size: ZrRuntimeViewportSizeV1) -> ZrRuntimeViewportSizeV1 {
    ZrRuntimeViewportSizeV1::new(size.width.max(1), size.height.max(1))
}

fn non_zero(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value.max(1)).expect("value is clamped to non-zero")
}

#[cfg(test)]
mod tests {
    use super::copy_rgba_to_xrgb;

    #[test]
    fn complete_rgba_frame_overwrites_the_surface_without_a_preclear() {
        let mut surface = [0x00ff_00ff, 0x00ff_00ff];

        let cleared = copy_rgba_to_xrgb(&mut surface, &[1, 2, 3, 255, 4, 5, 6, 255]);

        assert!(!cleared);
        assert_eq!(surface, [0x0001_0203, 0x0004_0506]);
    }

    #[test]
    fn truncated_rgba_frame_clears_uncovered_surface_pixels() {
        let mut surface = [0x00ff_00ff, 0x00ff_00ff];

        let cleared = copy_rgba_to_xrgb(&mut surface, &[1, 2, 3, 255]);

        assert!(cleared);
        assert_eq!(surface, [0x0001_0203, 0]);
    }
}
