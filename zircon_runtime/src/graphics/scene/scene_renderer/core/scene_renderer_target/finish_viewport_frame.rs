use crate::core::framework::render::RenderCaptureReport;
use crate::core::math::UVec2;
use crate::graphics::backend::RenderBackend;
use crate::graphics::types::{GraphicsError, ViewportFrame};
use crate::rhi::SubmissionPollReceipt;

pub(crate) fn finish_viewport_frame(
    backend: &RenderBackend,
    texture: &wgpu::Texture,
    size: UVec2,
    generation: u64,
    capture_report: RenderCaptureReport,
    observe_poll: &mut impl FnMut(SubmissionPollReceipt) -> Result<(), GraphicsError>,
) -> Result<ViewportFrame, GraphicsError> {
    let rgba = backend.read_product_diagnostic_texture_rgba8_blocking(
        generation,
        texture,
        size.x,
        size.y,
        "zircon-explicit-rgba8-capture",
        observe_poll,
    )?;

    Ok(ViewportFrame {
        width: size.x,
        height: size.y,
        rgba,
        generation,
        capture_report,
    })
}
