use crate::core::framework::render::RenderCaptureReport;
use crate::core::math::UVec2;
use crate::graphics::backend::read_texture_rgba;
use crate::graphics::types::{GraphicsError, ViewportFrame};

pub(crate) fn finish_viewport_frame(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    size: UVec2,
    generation: u64,
    capture_report: RenderCaptureReport,
) -> Result<ViewportFrame, GraphicsError> {
    let rgba = read_texture_rgba(device, queue, texture, size)?;

    Ok(ViewportFrame {
        width: size.x,
        height: size.y,
        rgba,
        generation,
        capture_report,
    })
}
