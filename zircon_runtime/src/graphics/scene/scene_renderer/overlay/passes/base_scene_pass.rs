use crate::core::framework::render::DisplayMode;

use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::mesh::{MeshDraw, MeshPipelineCache};
use crate::graphics::types::ViewportRenderFrame;

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
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("BaseScenePass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_bind_group(0, scene_bind_group, &[]);
        if frame.scene.overlays.display_mode == DisplayMode::WireOnly {
            return;
        }
        for draw in mesh_draws {
            let pipeline = mesh_pipelines.ensure_pipeline(device, streamer, draw.pipeline_key());
            pass.set_pipeline(pipeline);
            draw.bind_model(&mut pass);
            draw.bind_texture(&mut pass);
            draw.bind_geometry_buffers(&mut pass);
            draw.record_indexed_draw(&mut pass);
        }
    }
}
