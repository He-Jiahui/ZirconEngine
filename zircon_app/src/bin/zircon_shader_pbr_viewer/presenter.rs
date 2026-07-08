use std::num::NonZeroU32;
use std::sync::Arc;

use softbuffer::{Context, Surface};
use winit::window::Window;
use zircon_runtime::core::math::UVec2;
use zircon_runtime::graphics::ViewportFrame;

pub(crate) struct SoftbufferViewportPresenter {
    _context: Context<Arc<dyn Window>>,
    surface: Surface<Arc<dyn Window>, Arc<dyn Window>>,
    size: UVec2,
}

impl SoftbufferViewportPresenter {
    pub(crate) fn new(window: Arc<dyn Window>) -> Result<Self, softbuffer::SoftBufferError> {
        let context = Context::new(window.clone())?;
        let mut surface = Surface::new(&context, window.clone())?;
        let size = window_size(window.as_ref());
        resize_surface(&mut surface, size)?;
        Ok(Self {
            _context: context,
            surface,
            size,
        })
    }

    pub(crate) fn resize(&mut self, size: UVec2) -> Result<(), softbuffer::SoftBufferError> {
        let size = clamp_size(size);
        resize_surface(&mut self.surface, size)?;
        self.size = size;
        Ok(())
    }

    pub(crate) fn present(
        &mut self,
        frame: &ViewportFrame,
    ) -> Result<(), softbuffer::SoftBufferError> {
        let frame_size = UVec2::new(frame.width.max(1), frame.height.max(1));
        if self.size != frame_size {
            self.resize(frame_size)?;
        }

        let window = self.surface.window().clone();
        let mut buffer = self.surface.buffer_mut()?;
        buffer.fill(0);
        for (pixel, rgba) in buffer
            .iter_mut()
            .take((frame_size.x * frame_size.y) as usize)
            .zip(frame.rgba.chunks_exact(4))
        {
            let red = rgba[0] as u32;
            let green = rgba[1] as u32;
            let blue = rgba[2] as u32;
            *pixel = (red << 16) | (green << 8) | blue;
        }

        window.pre_present_notify();
        buffer.present()
    }
}

pub(crate) fn window_size(window: &dyn Window) -> UVec2 {
    let size = window.surface_size();
    UVec2::new(size.width.max(1), size.height.max(1))
}

fn resize_surface(
    surface: &mut Surface<Arc<dyn Window>, Arc<dyn Window>>,
    size: UVec2,
) -> Result<(), softbuffer::SoftBufferError> {
    surface.resize(non_zero(size.x), non_zero(size.y))
}

fn clamp_size(size: UVec2) -> UVec2 {
    UVec2::new(size.x.max(1), size.y.max(1))
}

fn non_zero(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value.max(1)).unwrap_or(NonZeroU32::MIN)
}
