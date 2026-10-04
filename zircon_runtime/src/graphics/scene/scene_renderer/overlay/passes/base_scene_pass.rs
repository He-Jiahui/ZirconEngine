use std::sync::Arc;

use wgpu::util::DeviceExt;

use crate::core::framework::render::DisplayMode;

use crate::graphics::pipeline::{PipelineAdmission, RenderPassStage};
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

/// 按调用方附件策略回放场景网格；直接路径和编译图共用此录制层，交互辅助层另行叠加。
pub(crate) struct BaseScenePass;

const BASE_SCENE_OPAQUE_PIPELINE_CONSUMER: &str = "base_scene_opaque";
const BASE_SCENE_TRANSPARENT_PIPELINE_CONSUMER: &str = "base_scene_transparent";

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
        light_grid_params_buffer: Option<wgpu::BufferBinding<'_>>,
        light_zbins_buffer: Option<wgpu::BufferBinding<'_>>,
        light_tile_masks_buffer: Option<wgpu::BufferBinding<'_>>,
        integrated_volumetric_view: Option<&wgpu::TextureView>,
        transmission_scene_color_view: Option<&wgpu::TextureView>,
        attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) -> MeshDrawReplayStats
    where
        I: IntoIterator<Item = MeshDrawCommandStream<'a>>,
        I::IntoIter: Clone,
    {
        self.record_commands_with_receiver_policy(
            encoder,
            device,
            color_view,
            depth_view,
            scene_bind_group,
            gpu_scene_bind_group,
            mesh_draw_commands,
            mesh_pipelines,
            streamer,
            frame,
            shadow_atlas_resources,
            render_region,
            light_grid_params_buffer,
            light_zbins_buffer,
            light_tile_masks_buffer,
            integrated_volumetric_view,
            transmission_scene_color_view,
            attachment_ops,
            depth_attachment_ops,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_environment_capture_commands_with_attachment_ops<'a, I>(
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
        render_region: ViewportRenderRegion,
        forward_receiver_bind_group: Option<&wgpu::BindGroup>,
        attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) -> MeshDrawReplayStats
    where
        I: IntoIterator<Item = MeshDrawCommandStream<'a>>,
        I::IntoIter: Clone,
    {
        self.record_commands_with_receiver_policy(
            encoder,
            device,
            color_view,
            depth_view,
            scene_bind_group,
            gpu_scene_bind_group,
            mesh_draw_commands,
            mesh_pipelines,
            streamer,
            frame,
            None,
            render_region,
            None,
            None,
            None,
            None,
            None,
            attachment_ops,
            depth_attachment_ops,
            forward_receiver_bind_group,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn record_commands_with_receiver_policy<'a, I>(
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
        light_grid_params_buffer: Option<wgpu::BufferBinding<'_>>,
        light_zbins_buffer: Option<wgpu::BufferBinding<'_>>,
        light_tile_masks_buffer: Option<wgpu::BufferBinding<'_>>,
        integrated_volumetric_view: Option<&wgpu::TextureView>,
        transmission_scene_color_view: Option<&wgpu::TextureView>,
        attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
        provided_forward_receiver_bind_group: Option<&wgpu::BindGroup>,
    ) -> MeshDrawReplayStats
    where
        I: IntoIterator<Item = MeshDrawCommandStream<'a>>,
        I::IntoIter: Clone,
    {
        if wire_only_load_store_can_skip(
            frame.overlays().display_mode,
            attachment_ops,
            depth_attachment_ops,
        ) {
            return MeshDrawReplayStats::default();
        }
        let mesh_draw_commands = mesh_draw_commands.into_iter();
        let needs_forward_receiver = !mesh_pipelines.environment_only_pbr_base_profile_enabled()
            || mesh_draw_commands.clone().any(|stream| {
                stream.commands().iter().any(|command| {
                    mesh_pipelines
                        .base_pipeline_requires_forward_receiver(command.pipeline_variant_id)
                })
            });
        // This stays alive through the render pass while avoiding the generic
        // forward-receiver allocation for the EnvironmentOnly-only path.
        let owned_forward_receiver_bind_group =
            (needs_forward_receiver && provided_forward_receiver_bind_group.is_none()).then(|| {
                mesh_pipelines.create_forward_shading_bind_group(
                    device,
                    frame,
                    render_region,
                    shadow_atlas_resources,
                    light_grid_params_buffer,
                    light_zbins_buffer,
                    light_tile_masks_buffer,
                    integrated_volumetric_view,
                    transmission_scene_color_view,
                )
            });
        let forward_shadow_receiver_bind_group =
            provided_forward_receiver_bind_group.or(owned_forward_receiver_bind_group.as_ref());
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
                    match mesh_pipelines.ensure_pipeline_admission_for_variant(
                        device,
                        streamer,
                        command.pipeline_variant_id,
                    ) {
                        PipelineAdmission::Ready(()) => {
                            mesh_pipelines.record_bound_mesh_pass_pipeline(
                                command.pipeline_kind,
                                command.pipeline_variant_id,
                            );
                            pass.set_pipeline(
                                mesh_pipelines
                                    .base_pipeline_for_ready_variant(command.pipeline_variant_id),
                            );
                        }
                        PipelineAdmission::Deferred(unavailable)
                        | PipelineAdmission::Failed(unavailable) => {
                            mesh_pipelines.record_base_pipeline_fallback_for_command(
                                command,
                                BASE_SCENE_OPAQUE_PIPELINE_CONSUMER,
                                unavailable,
                            );
                            replayer.invalidate_state_after_external_pipeline();
                            return false;
                        }
                    }
                }
                if mesh_pipelines
                    .base_pipeline_requires_forward_receiver(command.pipeline_variant_id)
                {
                    let Some(forward_shadow_receiver_bind_group) =
                        forward_shadow_receiver_bind_group
                    else {
                        replayer.invalidate_state_after_external_pipeline();
                        return false;
                    };
                    replayer.bind_forward_shadow_receiver_if_needed(
                        pass,
                        forward_shadow_receiver_bind_group,
                    );
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
        light_grid_params_buffer: Option<wgpu::BufferBinding<'_>>,
        light_zbins_buffer: Option<wgpu::BufferBinding<'_>>,
        light_tile_masks_buffer: Option<wgpu::BufferBinding<'_>>,
        integrated_volumetric_view: Option<&wgpu::TextureView>,
        transmission_scene_color_view: Option<&wgpu::TextureView>,
        attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) -> MeshDrawReplayStats {
        if wire_only_load_store_can_skip(
            frame.overlays().display_mode,
            attachment_ops,
            depth_attachment_ops,
        ) {
            return MeshDrawReplayStats::default();
        }
        let submission_order = build_transparent_submission_order(
            mesh_draw_commands,
            &frame.extract.sprites.phase_queue,
        );
        if submission_order.is_empty() {
            return MeshDrawReplayStats::default();
        }
        let transparent_sprites = prepare_transparent_sprite_draws(device, streamer, frame);
        let forward_shadow_receiver_bind_group = mesh_pipelines.create_forward_shading_bind_group(
            device,
            frame,
            render_region,
            shadow_atlas_resources,
            light_grid_params_buffer,
            light_zbins_buffer,
            light_tile_masks_buffer,
            integrated_volumetric_view,
            transmission_scene_color_view,
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
                            .pipeline_uses_builtin_fallback_shader(
                                streamer,
                                command.pipeline_key(),
                            );
                        if replayer
                            .should_set_pipeline(command.pipeline_kind, command.pipeline_variant_id)
                        {
                            match mesh_pipelines.ensure_pipeline_admission_for_variant(
                                device,
                                streamer,
                                command.pipeline_variant_id,
                            ) {
                                PipelineAdmission::Ready(()) => {
                                    mesh_pipelines.record_bound_mesh_pass_pipeline(
                                        command.pipeline_kind,
                                        command.pipeline_variant_id,
                                    );
                                    pass.set_pipeline(
                                        mesh_pipelines.base_pipeline_for_ready_variant(
                                            command.pipeline_variant_id,
                                        ),
                                    );
                                }
                                PipelineAdmission::Deferred(unavailable)
                                | PipelineAdmission::Failed(unavailable) => {
                                    mesh_pipelines.record_base_pipeline_fallback_for_command(
                                        command,
                                        BASE_SCENE_TRANSPARENT_PIPELINE_CONSUMER,
                                        unavailable,
                                    );
                                    replayer.invalidate_state_after_external_pipeline();
                                    return false;
                                }
                            }
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
                TransparentSubmissionSource::Sprite { sprite_index } => {
                    let Some(sprite_draw) = transparent_sprites
                        .get(sprite_index)
                        .and_then(Option::as_ref)
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

// 纯线框仍须执行清理或丢弃附件的 pass；只有颜色和深度都保留时，才可跳过整个场景 pass。
fn wire_only_load_store_can_skip(
    display_mode: DisplayMode,
    color_ops: RenderGraphAttachmentOps,
    depth_ops: RenderGraphAttachmentOps,
) -> bool {
    display_mode == DisplayMode::WireOnly
        && color_ops == RenderGraphAttachmentOps::load_store()
        && depth_ops == RenderGraphAttachmentOps::load_store()
}

struct PreparedTransparentSpriteDraw {
    texture: Arc<GpuTextureResource>,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
}

fn prepare_transparent_sprite_draws(
    device: &wgpu::Device,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
) -> Vec<Option<PreparedTransparentSpriteDraw>> {
    let mut draws = Vec::with_capacity(frame.sprites().len());
    draws.resize_with(frame.sprites().len(), || None);
    for (sprite_index, vertices) in build_sprite_vertices(frame, RenderPassStage::Transparent3d) {
        let Some(vertex_count) = u32::try_from(vertices.len()).ok() else {
            continue;
        };
        let Some(sprite) = frame.sprites().get(sprite_index) else {
            continue;
        };
        let Some(slot) = draws.get_mut(sprite_index) else {
            continue;
        };
        *slot = Some(PreparedTransparentSpriteDraw {
            texture: streamer.texture(Some(sprite.image.id())),
            vertex_buffer: create_sprite_vertex_buffer(device, &vertices),
            vertex_count,
        });
    }
    draws
}

fn create_sprite_vertex_buffer(device: &wgpu::Device, vertices: &[SpriteVertex]) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-transparent-sprite-vertices"),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::VERTEX,
    })
}

#[cfg(test)]
#[path = "tests/base_scene_pass.rs"]
mod tests;
