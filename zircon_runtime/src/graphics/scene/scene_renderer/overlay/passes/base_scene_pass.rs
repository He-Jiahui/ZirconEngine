use std::sync::Arc;

use wgpu::util::DeviceExt;

use crate::core::framework::render::DisplayMode;

use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::resources::{GpuTextureResource, ResourceStreamer};
use crate::graphics::scene::scene_renderer::attachment_ops::{
    color_attachment_operations, depth_attachment_operations,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshDrawCommand, MeshDrawCommandReplayer, MeshDrawCommandStream, MeshDrawReplayStats,
    MeshSceneDataBindHandle,
};
use crate::graphics::scene::scene_renderer::mesh::MeshPipelineCache;
use crate::graphics::scene::scene_renderer::shadow::atlas::ShadowAtlasResources;
use crate::graphics::scene::scene_renderer::sprite::{
    build_sprite_vertices, SpriteRenderer, SpriteVertex,
};
use crate::graphics::scene::scene_renderer::transparent::{
    build_transparent_submission_order, TransparentSubmissionSource,
};
use crate::graphics::types::{ViewportRenderFrame, ViewportRenderRegion};
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
        shadow_atlas_resources: Option<&ShadowAtlasResources>,
        render_region: ViewportRenderRegion,
        light_grid_params_buffer: Option<&wgpu::Buffer>,
        light_zbins_buffer: Option<&wgpu::Buffer>,
        light_tile_masks_buffer: Option<&wgpu::Buffer>,
        attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) -> MeshDrawReplayStats
    where
        I: IntoIterator<Item = MeshDrawCommandStream<'a>>,
    {
        let forward_shadow_receiver_bind_group = mesh_pipelines
            .create_forward_shadow_receiver_bind_group(
                device,
                shadow_atlas_resources,
                light_grid_params_buffer,
                light_zbins_buffer,
                light_tile_masks_buffer,
            );
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
        if !render_region.apply_physical_to_render_pass(&mut pass) {
            return MeshDrawReplayStats::default();
        }
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

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_transparent_mixed_with_attachment_ops<'a>(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        gpu_scene_bind_group: Option<MeshSceneDataBindHandle<'a>>,
        mesh_draw_commands: &'a [MeshDrawCommand],
        mesh_pipelines: &mut MeshPipelineCache,
        streamer: &ResourceStreamer,
        sprite_renderer: &SpriteRenderer,
        frame: &ViewportRenderFrame,
        shadow_atlas_resources: Option<&ShadowAtlasResources>,
        render_region: ViewportRenderRegion,
        light_grid_params_buffer: Option<&wgpu::Buffer>,
        light_zbins_buffer: Option<&wgpu::Buffer>,
        light_tile_masks_buffer: Option<&wgpu::Buffer>,
        attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) -> MeshDrawReplayStats {
        let submission_order = build_transparent_submission_order(
            mesh_draw_commands,
            &frame.extract.sprites.phase_queue,
        );
        if submission_order.is_empty() {
            return MeshDrawReplayStats::default();
        }
        let transparent_sprites = prepare_transparent_sprite_draws(device, streamer, frame);
        let forward_shadow_receiver_bind_group = mesh_pipelines
            .create_forward_shadow_receiver_bind_group(
                device,
                shadow_atlas_resources,
                light_grid_params_buffer,
                light_zbins_buffer,
                light_tile_masks_buffer,
            );
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("TransparentMixedScenePass"),
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
        if !render_region.apply_physical_to_render_pass(&mut pass) {
            return MeshDrawReplayStats::default();
        }
        pass.set_bind_group(0, scene_bind_group, &[]);
        pass.set_bind_group(1, &forward_shadow_receiver_bind_group, &[]);
        if frame.overlays().display_mode == DisplayMode::WireOnly {
            return MeshDrawReplayStats::default();
        }
        let mut replayer = MeshDrawCommandReplayer::default();
        for item in submission_order {
            match item.source {
                TransparentSubmissionSource::Mesh { command_index } => {
                    let Some(command) = mesh_draw_commands.get(command_index) else {
                        continue;
                    };
                    pass.set_bind_group(1, &forward_shadow_receiver_bind_group, &[]);
                    let stream = MeshDrawCommandStream::new(std::slice::from_ref(command), None);
                    replayer.replay_command_stream(&mut pass, stream, |replayer, pass, command| {
                        let uses_builtin_fallback_shader = mesh_pipelines
                            .pipeline_uses_builtin_fallback_shader(streamer, command.pipeline_key());
                        if replayer
                            .should_set_pipeline(command.pipeline_kind, command.pipeline_variant_id)
                        {
                            let pipeline = mesh_pipelines
                                .ensure_pipeline_for_variant(
                                    device,
                                    streamer,
                                    command.pipeline_variant_id,
                                )
                                .expect(
                                    "base mesh command must resolve a cache-backed pipeline variant",
                                );
                            pass.set_pipeline(pipeline);
                        }
                        replayer.bind_gpu_scene_if_needed(
                            pass,
                            command,
                            gpu_scene_bind_group,
                        );
                        if uses_builtin_fallback_shader {
                            replayer.bind_standard_material_if_needed(pass, command);
                        } else {
                            replayer.bind_material_if_needed(pass, command);
                        }
                        replayer.bind_geometry_if_needed(pass, command);
                        true
                    });
                }
                TransparentSubmissionSource::Sprite { sprite_index } => {
                    let Some(sprite_draw) = transparent_sprites
                        .iter()
                        .find(|draw| draw.sprite_index == sprite_index)
                    else {
                        continue;
                    };
                    sprite_renderer.record_vertices_in_pass(
                        &mut pass,
                        scene_bind_group,
                        &sprite_draw.texture.bind_group,
                        &sprite_draw.vertex_buffer,
                        sprite_draw.vertex_count,
                    );
                    replayer.invalidate_state_after_external_pipeline();
                }
            }
        }
        replayer.stats()
    }
}

struct PreparedTransparentSpriteDraw {
    sprite_index: usize,
    texture: Arc<GpuTextureResource>,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
}

fn prepare_transparent_sprite_draws(
    device: &wgpu::Device,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
) -> Vec<PreparedTransparentSpriteDraw> {
    build_sprite_vertices(frame, RenderPassStage::Transparent3d)
        .into_iter()
        .filter_map(|(sprite_index, vertices)| {
            let vertex_count = u32::try_from(vertices.len()).ok()?;
            let sprite = frame.sprites().get(sprite_index)?;
            let texture = streamer.texture(Some(sprite.image.id()));
            let vertex_buffer = create_sprite_vertex_buffer(device, &vertices);
            Some(PreparedTransparentSpriteDraw {
                sprite_index,
                texture,
                vertex_buffer,
                vertex_count,
            })
        })
        .collect()
}

fn create_sprite_vertex_buffer(device: &wgpu::Device, vertices: &[SpriteVertex]) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-transparent-sprite-vertices"),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::VERTEX,
    })
}
