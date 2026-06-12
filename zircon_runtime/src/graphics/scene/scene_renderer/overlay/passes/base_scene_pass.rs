use crate::core::framework::render::DisplayMode;

use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::attachment_ops::{
    color_attachment_operations, depth_attachment_operations,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshDrawCommandReplayer, MeshDrawCommandStream, MeshDrawReplayStats, MeshSceneDataBindHandle,
};
use crate::graphics::scene::scene_renderer::mesh::MeshPipelineCache;
use crate::graphics::scene::scene_renderer::primitives::SceneUniform;
use crate::graphics::types::ViewportRenderFrame;
use crate::render_graph::RenderGraphAttachmentOps;

pub(crate) struct BaseScenePass;

impl BaseScenePass {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_commands_with_attachment_ops<'a, I>(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        gpu_scene_bind_group: Option<MeshSceneDataBindHandle<'a>>,
        mesh_draw_commands: I,
        mesh_pipelines: &mut MeshPipelineCache,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        queue: Option<&wgpu::Queue>,
        shadow_map_view: Option<&wgpu::TextureView>,
        shadow_scene_uniform: Option<SceneUniform>,
        attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) -> MeshDrawReplayStats
    where
        I: IntoIterator<Item = MeshDrawCommandStream<'a>>,
    {
        if let Some(queue) = queue {
            mesh_pipelines.update_forward_shadow_receiver(queue, shadow_scene_uniform);
        }
        let forward_shadow_receiver_bind_group =
            mesh_pipelines.create_forward_shadow_receiver_bind_group(device, shadow_map_view);
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
        pass.set_bind_group(1, &forward_shadow_receiver_bind_group, &[]);
        if frame.overlays().display_mode == DisplayMode::WireOnly {
            return MeshDrawReplayStats::default();
        }
        let mut replayer = MeshDrawCommandReplayer::default();
        for stream in mesh_draw_commands {
            replayer.replay_command_stream(&mut pass, stream, |replayer, pass, command| {
                let uses_builtin_fallback_shader = mesh_pipelines
                    .pipeline_uses_builtin_fallback_shader(streamer, command.pipeline_key());
                if replayer.should_set_pipeline(command.pipeline_kind, command.pipeline_variant_id)
                {
                    let pipeline = mesh_pipelines
                        .ensure_pipeline_for_variant(device, streamer, command.pipeline_variant_id)
                        .expect("base mesh command must resolve a cache-backed pipeline variant");
                    pass.set_pipeline(pipeline);
                }
                replayer.bind_gpu_scene_if_needed(pass, command, gpu_scene_bind_group);
                if uses_builtin_fallback_shader {
                    replayer.bind_standard_material_if_needed(pass, command);
                } else {
                    replayer.bind_material_if_needed(pass, command);
                }
                replayer.bind_geometry_if_needed(pass, command);
                true
            });
        }
        replayer.stats()
    }
}
