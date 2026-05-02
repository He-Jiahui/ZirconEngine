use std::num::NonZeroU32;
use std::sync::Arc;

use softbuffer::{Context, Surface};
use winit::window::Window;
use zircon_runtime_interface::ZrRuntimeViewportSizeV1;

use crate::entry::runtime_library::RuntimeFrame;

pub(crate) struct SoftbufferRuntimePresenter {
    #[allow(dead_code)]
    context: Context<Arc<dyn Window>>,
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
            context,
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
        frame: &RuntimeFrame,
    ) -> Result<(), softbuffer::SoftBufferError> {
        let frame_size = ZrRuntimeViewportSizeV1::new(frame.width().max(1), frame.height().max(1));
        if self.size != frame_size {
            self.resize(frame_size)?;
        }

        let window = self.surface.window().clone();
        let mut buffer = self.surface.buffer_mut()?;
        buffer.fill(0);
        let pixel_count = (frame_size.width as usize) * (frame_size.height as usize);
        for (pixel, rgba) in buffer
            .iter_mut()
            .take(pixel_count)
            .zip(frame.rgba().chunks_exact(4))
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
