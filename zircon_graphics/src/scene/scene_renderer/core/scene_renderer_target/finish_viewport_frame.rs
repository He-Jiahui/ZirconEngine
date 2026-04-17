use crate::backend::{read_texture_rgba, OffscreenTarget};
use crate::types::{GraphicsError, ViewportFrame};

pub(crate) fn finish_viewport_frame(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    target: &OffscreenTarget,
    generation: u64,
) -> Result<ViewportFrame, GraphicsError> {
    let rgba = read_texture_rgba(device, queue, &target.final_color, target.size)?;

    Ok(ViewportFrame {
        width: target.size.x,
        height: target.size.y,
        rgba,
        generation,
    })
}
