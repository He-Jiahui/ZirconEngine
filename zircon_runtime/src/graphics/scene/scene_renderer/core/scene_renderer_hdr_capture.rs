use crate::core::framework::render::{decode_rgba16f_texels, RenderSceneSnapshot};
use crate::core::math::UVec2;
use crate::graphics::backend::{read_texture_rgba16float_region, Rgba16FloatTextureRegionReadback};
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

use super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    /// Renders the scene through the normal HDR scene path and reads back the
    /// linear pre-output-transfer color buffer for offline capture workflows.
    pub fn render_scene_color_hdr(
        &mut self,
        snapshot: RenderSceneSnapshot,
        viewport_size: impl Into<UVec2>,
    ) -> Result<Vec<[f32; 4]>, GraphicsError> {
        let frame = ViewportRenderFrame::from_snapshot(snapshot, viewport_size);
        self.render_frame_to_offscreen_target(&frame)?;
        let target = self.target.as_ref().expect("offscreen target");
        let bytes = read_texture_rgba16float_region(
            &self.backend.device,
            &self.backend.queue,
            &target.scene_color,
            Rgba16FloatTextureRegionReadback {
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                size: wgpu::Extent3d {
                    width: target.render_size.x,
                    height: target.render_size.y,
                    depth_or_array_layers: 1,
                },
                label: "zircon-scene-color-hdr-capture",
            },
        )?;
        Ok(decode_rgba16f_texels(&bytes))
    }
}
