use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::overlay::PreparedOverlayBuffers;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

use super::super::super::scene_renderer_core::SceneRendererCore;

pub(super) fn prepare_overlay_buffers(
    renderer: &mut SceneRendererCore,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
) -> Result<PreparedOverlayBuffers, GraphicsError> {
    renderer.overlay_renderer.prepare_buffers(
        device,
        queue,
        &renderer.texture_bind_group_layout,
        streamer,
        frame,
    )
}
