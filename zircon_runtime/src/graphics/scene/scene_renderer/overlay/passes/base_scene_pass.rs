use crate::core::framework::render::DisplayMode;

use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::mesh::{MeshDraw, MeshPipelineCache};
use crate::graphics::types::EditorOrRuntimeFrame;

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
        frame: &EditorOrRuntimeFrame,
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
            let pipeline = mesh_pipelines.ensure_pipeline(device, streamer, &draw.pipeline_key);
            pass.set_pipeline(pipeline);
            pass.set_bind_group(1, &draw.model_bind_group, &[]);
            pass.set_bind_group(2, &draw.texture.bind_group, &[]);
            pass.set_vertex_buffer(0, draw.mesh.vertex_buffer.slice(..));
            pass.set_index_buffer(draw.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            if let Some(indirect_args_buffer) = &draw.indirect_args_buffer {
                pass.draw_indexed_indirect(indirect_args_buffer, draw.indirect_args_offset);
            } else {
                pass.draw_indexed(
                    draw.first_index..(draw.first_index + draw.draw_index_count),
                    0,
                    0..1,
                );
            }
        }
    }
}
