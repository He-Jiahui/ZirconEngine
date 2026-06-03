use crate::core::framework::render::DisplayMode;

use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::attachment_ops::{
    color_attachment_operations, depth_attachment_operations,
};
use crate::graphics::scene::scene_renderer::mesh::{MeshDraw, MeshPipelineCache};
use crate::graphics::types::ViewportRenderFrame;
use crate::render_graph::RenderGraphAttachmentOps;

pub(crate) struct BaseScenePass;

impl BaseScenePass {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record<'a, I>(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        mesh_draws: I,
        mesh_pipelines: &mut MeshPipelineCache,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
    ) where
        I: IntoIterator<Item = &'a MeshDraw>,
    {
        self.record_with_attachment_ops(
            encoder,
            device,
            color_view,
            depth_view,
            scene_bind_group,
            mesh_draws,
            mesh_pipelines,
            streamer,
            frame,
            RenderGraphAttachmentOps::load_store(),
            RenderGraphAttachmentOps::load_store(),
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_with_attachment_ops<'a, I>(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        mesh_draws: I,
        mesh_pipelines: &mut MeshPipelineCache,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) where
        I: IntoIterator<Item = &'a MeshDraw>,
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("BaseScenePass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(attachment_ops, wgpu::Color::TRANSPARENT),
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(depth_attachment_operations(depth_attachment_ops, 1.0)),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_bind_group(0, scene_bind_group, &[]);
        if frame.overlays().display_mode == DisplayMode::WireOnly {
            return;
        }
        for draw in mesh_draws {
            let pipeline = mesh_pipelines.ensure_pipeline(device, streamer, draw.pipeline_key());
            pass.set_pipeline(pipeline);
            draw.bind_model(&mut pass);
            draw.bind_texture(&mut pass);
            draw.bind_material(&mut pass);
            draw.bind_geometry_buffers(&mut pass);
            draw.record_indexed_draw(&mut pass);
        }
    }
}
