use crate::backend::OffscreenTarget;
use crate::scene::{ResourceStreamer, SceneRendererCore, OFFSCREEN_FORMAT};
use crate::types::{EditorOrRuntimeFrame, GraphicsError, ViewportFrameTextureHandle};

use super::shared_texture_offscreen_renderer::SharedTextureOffscreenRenderer;

impl SharedTextureOffscreenRenderer {
    pub(in crate::service) fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene_renderer: &mut SceneRendererCore,
        streamer: &ResourceStreamer,
        frame: &EditorOrRuntimeFrame,
        generation: u64,
    ) -> Result<ViewportFrameTextureHandle, GraphicsError> {
        let size =
            zircon_math::UVec2::new(frame.viewport.size.x.max(1), frame.viewport.size.y.max(1));
        if self
            .target
            .as_ref()
            .is_none_or(|target| target.size != size)
        {
            self.target = Some(OffscreenTarget::new(device, size));
        }
        let target = self.target.as_ref().unwrap();

        scene_renderer.render_scene(
            device,
            queue,
            streamer,
            frame,
            &target.final_color_view,
            &target.depth_view,
        )?;

        Ok(ViewportFrameTextureHandle {
            width: target.size.x,
            height: target.size.y,
            texture: target.final_color.clone(),
            format: OFFSCREEN_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            generation,
        })
    }
}
