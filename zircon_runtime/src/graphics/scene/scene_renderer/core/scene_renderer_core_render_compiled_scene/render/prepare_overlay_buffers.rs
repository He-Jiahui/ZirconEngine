use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::overlay::PreparedOverlayBuffers;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use zr_rhi_wgpu::WgpuTextureUploadBatch;

use super::super::super::scene_renderer_core::SceneRendererCore;

pub(super) fn prepare_overlay_buffers(
    renderer: &mut SceneRendererCore,
    device: &wgpu::Device,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    frame_texture_uploads: &mut WgpuTextureUploadBatch,
) -> Result<PreparedOverlayBuffers, GraphicsError> {
    renderer.overlay_renderer.prepare_buffers(
        device,
        &renderer.texture_bind_group_layout,
        streamer,
        frame,
        frame_texture_uploads,
    )
}
