use std::collections::BTreeSet;

use bytemuck::bytes_of;

use crate::core::framework::scene::EntityId;
use crate::core::math::{is_finite_mat4, Mat4};
use crate::graphics::scene::resources::GpuMeshVertex;
use crate::graphics::scene::scene_renderer::attachment_ops::depth_attachment_operations;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshDrawCommandReplayer, MeshDrawCommandStream, MeshDrawReplayStats, MeshPassPipelineKind,
    MeshPipelineVariantId, MeshSceneDataBindHandle, GPU_SCENE_BIND_GROUP_SLOT,
};
use crate::graphics::scene::scene_renderer::primitives::SceneUniform;
use crate::graphics::types::ViewportRenderFrame;
use crate::graphics::visibility::VisibilityViewKey;
use crate::render_graph::RenderGraphAttachmentOps;

use super::plan::ShadowAtlasSlotPass;
use super::shadow_map_shader_source::SHADOW_MAP_SHADER;

const SHADOW_DEPTH_BIAS_CONSTANT: i32 = 2;
const SHADOW_DEPTH_BIAS_SLOPE_SCALE: f32 = 2.0;
const SHADOW_DEPTH_BIAS_CLAMP: f32 = 0.0;

pub(crate) struct ShadowMapRenderer {
    pipeline: wgpu::RenderPipeline,
    alpha_mask_pipeline: wgpu::RenderPipeline,
    scene_uniform_buffer: wgpu::Buffer,
    scene_bind_group: wgpu::BindGroup,
}

impl ShadowMapRenderer {
    pub(crate) fn new(
        device: &wgpu::Device,
        scene_layout: &wgpu::BindGroupLayout,
        material_layout: &wgpu::BindGroupLayout,
        gpu_scene_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-shadow-map-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADOW_MAP_SHADER.into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-shadow-map-layout"),
            bind_group_layouts: &[Some(scene_layout), None, None, Some(gpu_scene_layout)],
            immediate_size: 0,
        });
        debug_assert_eq!(GPU_SCENE_BIND_GROUP_SLOT, 3);
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("zircon-shadow-map-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[GpuMeshVertex::layout()],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: super::super::core::DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: SHADOW_DEPTH_BIAS_CONSTANT,
                    slope_scale: SHADOW_DEPTH_BIAS_SLOPE_SCALE,
                    clamp: SHADOW_DEPTH_BIAS_CLAMP,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: None,
            multiview_mask: None,
            cache: None,
        });
        let alpha_mask_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-shadow-map-alpha-mask-layout"),
            bind_group_layouts: &[
                Some(scene_layout),
                None,
                Some(material_layout),
                Some(gpu_scene_layout),
            ],
            immediate_size: 0,
        });
        let alpha_mask_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("zircon-shadow-map-alpha-mask-pipeline"),
            layout: Some(&alpha_mask_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[GpuMeshVertex::layout()],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: super::super::core::DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: SHADOW_DEPTH_BIAS_CONSTANT,
                    slope_scale: SHADOW_DEPTH_BIAS_SLOPE_SCALE,
                    clamp: SHADOW_DEPTH_BIAS_CLAMP,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_alpha_mask"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[],
            }),
            multiview_mask: None,
            cache: None,
        });
        let scene_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-shadow-map-scene-uniform"),
            size: std::mem::size_of::<SceneUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-shadow-map-scene-bind-group"),
            layout: scene_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scene_uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            pipeline,
            alpha_mask_pipeline,
            scene_uniform_buffer,
            scene_bind_group,
        }
    }

    pub(crate) fn record_atlas_commands_with_attachment_ops<'a>(
        &self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        pass_name: &str,
        atlas_view: &wgpu::TextureView,
        slot_passes: &[ShadowAtlasSlotPass],
        frame: &ViewportRenderFrame,
        gpu_scene_bind_group: Option<MeshSceneDataBindHandle<'a>>,
        mesh_draw_commands: MeshDrawCommandStream<'a>,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> MeshDrawReplayStats {
        if slot_passes.is_empty() || mesh_draw_commands.is_empty() {
            record_depth_only_pass(encoder, pass_name, atlas_view, attachment_ops);
            return MeshDrawReplayStats::default();
        }

        let mut wrote_first_slot = false;
        let mut combined = MeshDrawReplayStats::default();
        for slot_pass in slot_passes {
            if slot_pass.rect.width == 0 || slot_pass.rect.height == 0 {
                continue;
            }
            let scene_uniform = scene_uniform_for_view_projection(slot_pass.view_proj);
            queue.write_buffer(&self.scene_uniform_buffer, 0, bytes_of(&scene_uniform));

            let atlas_attachment_ops = if wrote_first_slot {
                RenderGraphAttachmentOps::load_store()
            } else {
                attachment_ops
            };
            wrote_first_slot = true;
            let slot_pass_name = format!("{pass_name}.atlas-slot-{}", slot_pass.slot_index);
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&slot_pass_name),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: atlas_view,
                    depth_ops: Some(depth_attachment_operations(atlas_attachment_ops, 1.0)),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_viewport(
                slot_pass.rect.x as f32,
                slot_pass.rect.y as f32,
                slot_pass.rect.width as f32,
                slot_pass.rect.height as f32,
                0.0,
                1.0,
            );
            pass.set_scissor_rect(
                slot_pass.rect.x,
                slot_pass.rect.y,
                slot_pass.rect.width,
                slot_pass.rect.height,
            );
            pass.set_bind_group(0, &self.scene_bind_group, &[]);
            let visible_entities = slot_pass
                .view_key
                .and_then(|view_key| visible_shadow_entities_for_view(frame, &view_key));
            add_replay_stats(
                &mut combined,
                self.replay_shadow_command_stream(
                    &mut pass,
                    gpu_scene_bind_group,
                    mesh_draw_commands,
                    visible_entities.as_ref(),
                ),
            );
        }

        if !wrote_first_slot {
            record_depth_only_pass(encoder, pass_name, atlas_view, attachment_ops);
        }
        combined
    }

    fn replay_shadow_command_stream<'pass>(
        &self,
        pass: &mut wgpu::RenderPass<'pass>,
        gpu_scene_bind_group: Option<MeshSceneDataBindHandle<'pass>>,
        mesh_draw_commands: MeshDrawCommandStream<'pass>,
        visible_entities: Option<&BTreeSet<EntityId>>,
    ) -> MeshDrawReplayStats {
        let mut replayer = MeshDrawCommandReplayer::default();
        let fixed_shadow_variant = MeshPipelineVariantId::new(0);
        replayer.replay_command_stream(pass, mesh_draw_commands, |replayer, pass, command| {
            if visible_entities.is_some_and(|entities| !entities.contains(&command.source_entity)) {
                return false;
            }
            match command.pipeline_kind {
                MeshPassPipelineKind::ShadowDepthAlphaMask => {
                    if replayer.should_set_pipeline(command.pipeline_kind, fixed_shadow_variant) {
                        pass.set_pipeline(&self.alpha_mask_pipeline);
                    }
                    replayer.bind_standard_material_if_needed(pass, command);
                }
                MeshPassPipelineKind::ShadowDepth => {
                    if replayer.should_set_pipeline(command.pipeline_kind, fixed_shadow_variant) {
                        pass.set_pipeline(&self.pipeline);
                    }
                }
                _ => return false,
            }
            replayer.bind_gpu_scene_if_needed(pass, command, gpu_scene_bind_group);
            replayer.bind_geometry_if_needed(pass, command);
            true
        });
        replayer.stats()
    }
}

fn visible_shadow_entities_for_view(
    frame: &ViewportRenderFrame,
    view_key: &VisibilityViewKey,
) -> Option<BTreeSet<EntityId>> {
    let frame_visibility = frame.frame_visibility()?;
    frame_visibility.view(view_key)?;
    Some(frame_visibility.visible_entity_set_for_view(view_key))
}

#[cfg(test)]
fn filter_shadow_commands_for_visible_entities(
    commands: &[crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshDrawCommand],
    visible_entities: &BTreeSet<EntityId>,
) -> Vec<crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshDrawCommand> {
    commands
        .iter()
        .filter(|command| visible_entities.contains(&command.source_entity))
        .cloned()
        .collect()
}

fn scene_uniform_for_view_projection(view_proj: Mat4) -> SceneUniform {
    let view_proj = finite_mat4_or_identity(view_proj);
    let view_proj_cols = view_proj.to_cols_array_2d();
    SceneUniform {
        view_proj: view_proj_cols,
        inverse_view_proj: finite_mat4_or_identity(view_proj.inverse()).to_cols_array_2d(),
        ambient_color: [0.0, 0.0, 0.0, 1.0],
        previous_view_proj: view_proj_cols,
        motion_params: [0.0, 0.0, 0.0, 0.0],
    }
}

fn record_depth_only_pass(
    encoder: &mut wgpu::CommandEncoder,
    pass_name: &str,
    depth_view: &wgpu::TextureView,
    attachment_ops: RenderGraphAttachmentOps,
) {
    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(pass_name),
        color_attachments: &[],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth_view,
            depth_ops: Some(depth_attachment_operations(attachment_ops, 1.0)),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
}

fn add_replay_stats(total: &mut MeshDrawReplayStats, next: MeshDrawReplayStats) {
    total.draw_call_count = total.draw_call_count.saturating_add(next.draw_call_count);
    total.state_change_count = total
        .state_change_count
        .saturating_add(next.state_change_count);
    total.bind_skip_count = total.bind_skip_count.saturating_add(next.bind_skip_count);
}

fn finite_mat4_or_identity(value: Mat4) -> Mat4 {
    if is_finite_mat4(value) {
        value
    } else {
        Mat4::IDENTITY
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::core::framework::render::RenderPhase;
    use crate::graphics::scene::resources::default_pipeline_key;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
        DrawInstanceSource, MeshDrawArgs, MeshDrawCommand, MeshGeometryHandle,
        MeshPassPipelineKind, MeshPipelineVariantId,
    };

    #[test]
    fn shadow_map_shader_keeps_opaque_depth_path_and_alpha_mask_cutoff_path() {
        assert!(super::SHADOW_MAP_SHADER.contains("@vertex"));
        assert!(super::SHADOW_MAP_SHADER.contains("fn fs_alpha_mask"));
        assert!(super::SHADOW_MAP_SHADER.contains(
            "@group(2) @binding(10) var<uniform> material_properties: MaterialPropertyUniform;"
        ));
        assert!(super::SHADOW_MAP_SHADER.contains("@location(7) uv1: vec2<f32>"));
        assert!(super::SHADOW_MAP_SHADER.contains(
            "let base_color_uv = transform_material_uv_channel(input.uv, input.uv1, material_properties.data2, material_properties.data7.x);"
        ));
        assert!(super::SHADOW_MAP_SHADER.contains("discard"));
        assert!(super::SHADOW_MAP_SHADER.contains("shadow_params.y"));
    }

    #[test]
    fn shadow_atlas_view_filter_keeps_only_visible_source_entities() {
        let commands = vec![test_command(11), test_command(22), test_command(33)];
        let visible_entities = [22, 33].into_iter().collect::<BTreeSet<_>>();

        let filtered =
            super::filter_shadow_commands_for_visible_entities(&commands, &visible_entities);

        assert_eq!(
            filtered
                .iter()
                .map(|command| command.source_entity)
                .collect::<Vec<_>>(),
            vec![22, 33]
        );
    }

    fn test_command(source_entity: u64) -> MeshDrawCommand {
        MeshDrawCommand::new(
            RenderPhase::Shadow,
            MeshPassPipelineKind::ShadowDepth,
            default_pipeline_key(),
            MeshPipelineVariantId::new(0),
            source_entity,
            DrawInstanceSource::GpuSceneInstance {
                first_instance_index: 0,
                instance_count: 1,
            },
            MeshGeometryHandle::test(source_entity),
            MeshDrawArgs::direct_indexed(0, 3),
        )
        .with_source_entity(source_entity)
    }
}
