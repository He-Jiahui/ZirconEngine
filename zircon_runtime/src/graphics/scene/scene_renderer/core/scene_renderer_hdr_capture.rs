use crate::core::framework::render::{
    decode_rgba16f_texels, RenderGraphTransientPoolReport, RenderSceneSnapshot,
};
use crate::core::math::UVec2;
use crate::graphics::backend::{read_texture_rgba16float_region, Rgba16FloatTextureRegionReadback};
use crate::graphics::scene::scene_renderer::core::{
    DEPTH_FORMAT, FINAL_COLOR_FORMAT, SCENE_COLOR_HDR_FORMAT,
};
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};

use super::scene_renderer::SceneRenderer;
use super::scene_renderer_runtime_outputs::reset_last_runtime_outputs;

impl SceneRenderer {
    /// Renders the scene through the normal HDR scene path and reads back the
    /// linear pre-output-transfer color buffer for offline capture workflows.
    pub fn render_scene_color_hdr(
        &mut self,
        snapshot: RenderSceneSnapshot,
        viewport_size: impl Into<UVec2>,
    ) -> Result<Vec<[f32; 4]>, GraphicsError> {
        let viewport_size = viewport_size.into();
        let frame = ViewportRenderFrame::from_snapshot(snapshot, viewport_size);
        reset_last_runtime_outputs(self);
        self.streamer.ensure_scene_resources(
            &self.backend.device,
            &self.backend.queue,
            &self.core.texture_bind_group_layout,
            &frame,
        )?;

        let scene_desc = capture_texture_desc(
            "zircon-hdr-capture-scene-color",
            viewport_size,
            TextureFormat::Rgba16Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED | TextureUsage::COPY_SRC,
        );
        let final_desc = capture_texture_desc(
            "zircon-hdr-capture-final-color",
            viewport_size,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        );
        let depth_desc = capture_texture_desc(
            "zircon-hdr-capture-depth",
            viewport_size,
            TextureFormat::Depth32Float,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED,
        );

        self.core.transient_resource_pool.begin_frame();
        let scene_color = self
            .core
            .transient_resource_pool
            .acquire_texture(&self.backend.device, &scene_desc);
        let final_color = self
            .core
            .transient_resource_pool
            .acquire_texture(&self.backend.device, &final_desc);
        let depth = self
            .core
            .transient_resource_pool
            .acquire_texture(&self.backend.device, &depth_desc);
        let scene_color_view = scene_color.create_view(&wgpu::TextureViewDescriptor::default());
        let final_color_view = final_color.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());

        let result = (|| {
            debug_assert_eq!(self.core.scene_color_format, SCENE_COLOR_HDR_FORMAT);
            debug_assert_eq!(self.core.final_color_format, FINAL_COLOR_FORMAT);
            debug_assert_eq!(self.core.depth_format, DEPTH_FORMAT);
            self.core.render_scene(
                &self.backend.device,
                &self.backend.queue,
                &self.streamer,
                &frame,
                &scene_color_view,
                &final_color_view,
                &depth_view,
            )?;
            let bytes = read_texture_rgba16float_region(
                &self.backend.device,
                &self.backend.queue,
                &scene_color,
                Rgba16FloatTextureRegionReadback {
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    size: wgpu::Extent3d {
                        width: viewport_size.x,
                        height: viewport_size.y,
                        depth_or_array_layers: 1,
                    },
                    label: "zircon-scene-color-hdr-capture",
                },
            )?;
            Ok(decode_rgba16f_texels(&bytes))
        })();

        drop((scene_color_view, final_color_view, depth_view));
        self.core
            .transient_resource_pool
            .release_texture(scene_desc, scene_color);
        self.core
            .transient_resource_pool
            .release_texture(final_desc, final_color);
        self.core
            .transient_resource_pool
            .release_texture(depth_desc, depth);
        self.core.transient_resource_pool.end_frame();
        if result.is_ok() {
            self.generation = self.generation.saturating_add(1);
        }
        result
    }

    pub fn last_transient_resource_pool_report(&self) -> RenderGraphTransientPoolReport {
        self.core.transient_resource_pool.last_frame_report()
    }
}

fn capture_texture_desc(
    label: &str,
    size: UVec2,
    format: TextureFormat,
    usage: TextureUsage,
) -> TextureDesc {
    TextureDesc::new(label, size.x.max(1), size.y.max(1), format, usage)
}
