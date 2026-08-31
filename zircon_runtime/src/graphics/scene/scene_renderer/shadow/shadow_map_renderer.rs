use std::collections::BTreeSet;
use std::mem::size_of;
use std::num::NonZeroU64;
use std::sync::Arc;

use bytemuck::bytes_of;

use crate::core::framework::scene::EntityId;
use crate::core::math::{Mat4, is_finite_mat4};
use crate::graphics::pipeline::PipelineAdmission;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::attachment_ops::depth_attachment_operations;
use crate::graphics::scene::scene_renderer::mesh::MeshPipelineCache;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshDrawCommandReplayer, MeshDrawCommandStream, MeshDrawReplayStats, MeshPassPipelineKind,
    MeshSceneDataBindHandle,
};
use crate::graphics::scene::scene_renderer::primitives::SceneUniform;
use crate::graphics::types::ViewportRenderFrame;
use crate::graphics::visibility::VisibilityViewKey;
use crate::render_graph::RenderGraphAttachmentOps;
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

use super::plan::ShadowAtlasSlotPass;

const SHADOW_ATLAS_PIPELINE_CONSUMER: &str = "shadow_atlas";

pub(crate) struct ShadowMapRenderer {
    scene_layout: wgpu::BindGroupLayout,
    environment: ShadowSceneEnvironmentBindingLease,
    slot_scene_workspace: ShadowSlotSceneWorkspace,
}

#[derive(Default)]
struct ShadowSlotSceneWorkspace {
    buffer: Option<wgpu::Buffer>,
    bind_groups: Vec<wgpu::BindGroup>,
    uniform_stride: u64,
}

pub(in crate::graphics::scene::scene_renderer) struct ShadowSceneEnvironmentBindingLease {
    _black_cube_texture: wgpu::Texture,
    black_cube_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    _brdf_lut_texture: wgpu::Texture,
    brdf_lut_view: wgpu::TextureView,
    sh9_buffer: wgpu::Buffer,
}

impl ShadowSceneEnvironmentBindingLease {
    pub(in crate::graphics::scene::scene_renderer) fn new(
        black_cube_texture: wgpu::Texture,
        black_cube_view: wgpu::TextureView,
        sampler: wgpu::Sampler,
        brdf_lut_texture: wgpu::Texture,
        brdf_lut_view: wgpu::TextureView,
        sh9_buffer: wgpu::Buffer,
    ) -> Self {
        Self {
            _black_cube_texture: black_cube_texture,
            black_cube_view,
            sampler,
            _brdf_lut_texture: brdf_lut_texture,
            brdf_lut_view,
            sh9_buffer,
        }
    }
}

impl ShadowMapRenderer {
    pub(in crate::graphics::scene::scene_renderer) fn new(
        scene_layout: &wgpu::BindGroupLayout,
        environment: ShadowSceneEnvironmentBindingLease,
    ) -> Self {
        Self {
            scene_layout: scene_layout.clone(),
            environment,
            slot_scene_workspace: ShadowSlotSceneWorkspace::default(),
        }
    }

    pub(crate) fn prepare_slot_scene_uploads(
        &mut self,
        device: &wgpu::Device,
        slot_passes: &[ShadowAtlasSlotPass],
    ) -> Result<WgpuBufferUploadBatch, String> {
        let mut uploads = WgpuBufferUploadBatch::new();
        if slot_passes.is_empty() {
            return Ok(uploads);
        }

        self.ensure_slot_scene_capacity(device, slot_passes.len())?;
        let stride = usize::try_from(self.slot_scene_workspace.uniform_stride).map_err(|_| {
            "shadow slot scene uniform stride exceeds host address space".to_owned()
        })?;
        let payload_len = stride
            .checked_mul(slot_passes.len())
            .ok_or_else(|| "shadow slot scene payload size overflow".to_owned())?;
        let mut payload = vec![0; payload_len];
        for (slot_ordinal, slot_pass) in slot_passes.iter().enumerate() {
            let scene_uniform = scene_uniform_for_view_projection(slot_pass.view_proj);
            let bytes = bytes_of(&scene_uniform);
            let start = slot_ordinal
                .checked_mul(stride)
                .expect("validated shadow slot payload offset must fit usize");
            payload[start..start + bytes.len()].copy_from_slice(bytes);
        }

        let payload: Arc<[u8]> = Arc::from(payload);
        let buffer = self
            .slot_scene_workspace
            .buffer
            .as_ref()
            .expect("non-empty shadow slot preparation must materialize its workspace")
            .clone();
        uploads.push(
            WgpuBufferUpload::new(buffer, 0, payload, 0..payload_len)
                .ok_or_else(|| "shadow slot scene upload range escaped its payload".to_owned())?,
        );
        Ok(uploads)
    }

    fn ensure_slot_scene_capacity(
        &mut self,
        device: &wgpu::Device,
        required_slots: usize,
    ) -> Result<(), String> {
        if required_slots <= self.slot_scene_workspace.bind_groups.len() {
            return Ok(());
        }
        let capacity = required_slots
            .checked_next_power_of_two()
            .ok_or_else(|| "shadow slot scene workspace capacity overflow".to_owned())?;
        let uniform_size = size_of::<SceneUniform>() as u64;
        let alignment = u64::from(device.limits().min_uniform_buffer_offset_alignment.max(1));
        let uniform_stride = align_up_u64(uniform_size, alignment)
            .ok_or_else(|| "shadow slot scene uniform alignment overflow".to_owned())?;
        let buffer_size = uniform_stride
            .checked_mul(capacity as u64)
            .ok_or_else(|| "shadow slot scene workspace byte size overflow".to_owned())?;
        if buffer_size > device.limits().max_buffer_size {
            return Err(format!(
                "shadow slot scene workspace requires {buffer_size} bytes but device limit is {}",
                device.limits().max_buffer_size
            ));
        }

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-shadow-slot-scene-uniform-workspace"),
            size: buffer_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let binding_size =
            NonZeroU64::new(uniform_size).expect("SceneUniform must have a non-zero binding size");
        let bind_groups = (0..capacity)
            .map(|slot_ordinal| {
                create_slot_scene_bind_group(
                    device,
                    &self.scene_layout,
                    &self.environment,
                    &buffer,
                    slot_ordinal as u64 * uniform_stride,
                    binding_size,
                )
            })
            .collect();

        self.slot_scene_workspace = ShadowSlotSceneWorkspace {
            buffer: Some(buffer),
            bind_groups,
            uniform_stride,
        };
        Ok(())
    }

    pub(crate) fn record_atlas_commands_with_attachment_ops<'a>(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        pass_name: &str,
        atlas_view: &wgpu::TextureView,
        mesh_pipelines: &mut MeshPipelineCache,
        streamer: &ResourceStreamer,
        slot_passes: &[ShadowAtlasSlotPass],
        frame: &ViewportRenderFrame,
        gpu_scene_bind_group: Option<MeshSceneDataBindHandle<'a>>,
        mesh_draw_commands: MeshDrawCommandStream<'a>,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<MeshDrawReplayStats, String> {
        if slot_passes.is_empty() || mesh_draw_commands.is_empty() {
            record_depth_only_pass(encoder, pass_name, atlas_view, attachment_ops);
            return Ok(MeshDrawReplayStats::default());
        }

        let forward_shadow_receiver_bind_group = mesh_pipelines
            .create_forward_shadow_receiver_bind_group(device, None, None, None, None);
        let mut wrote_first_slot = false;
        let mut combined = MeshDrawReplayStats::default();
        for (slot_ordinal, slot_pass) in slot_passes.iter().enumerate() {
            if slot_pass.rect.width == 0 || slot_pass.rect.height == 0 {
                continue;
            }
            let scene_bind_group = self
                .slot_scene_workspace
                .bind_groups
                .get(slot_ordinal)
                .ok_or_else(|| {
                    format!(
                        "shadow slot scene workspace prepared {} bindings but pass ordinal {slot_ordinal} was requested",
                        self.slot_scene_workspace.bind_groups.len()
                    )
                })?;

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
            pass.set_bind_group(0, scene_bind_group, &[]);
            pass.set_bind_group(1, &forward_shadow_receiver_bind_group, &[]);
            let visible_entities = slot_pass
                .view_key
                .and_then(|view_key| visible_shadow_entities_for_view(frame, &view_key));
            add_replay_stats(
                &mut combined,
                self.replay_shadow_command_stream(
                    device,
                    mesh_pipelines,
                    streamer,
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
        Ok(combined)
    }

    fn replay_shadow_command_stream<'pass>(
        &self,
        device: &wgpu::Device,
        mesh_pipelines: &mut MeshPipelineCache,
        streamer: &ResourceStreamer,
        pass: &mut wgpu::RenderPass<'pass>,
        gpu_scene_bind_group: Option<MeshSceneDataBindHandle<'pass>>,
        mesh_draw_commands: MeshDrawCommandStream<'pass>,
        visible_entities: Option<&BTreeSet<EntityId>>,
    ) -> MeshDrawReplayStats {
        let mesh_draw_commands = if visible_entities.is_some() {
            // Global indirect batches span commands from multiple entities. Until shadow
            // visibility owns view-local compacted ranges, filtering must stay per command.
            mesh_draw_commands.without_indirect()
        } else {
            mesh_draw_commands
        };
        let mut replayer = MeshDrawCommandReplayer::default();
        replayer.replay_command_stream(pass, mesh_draw_commands, |replayer, pass, command| {
            if visible_entities.is_some_and(|entities| !entities.contains(&command.source_entity)) {
                return false;
            }
            let kind = match command.pipeline_kind {
                MeshPassPipelineKind::ShadowDepthAlphaMask | MeshPassPipelineKind::ShadowDepth => {
                    command.pipeline_kind
                }
                _ => return false,
            };
            if replayer.should_set_pipeline(kind, command.pipeline_variant_id) {
                match mesh_pipelines.ensure_shadow_pipeline_admission_for_variant(
                    device,
                    streamer,
                    kind,
                    command.pipeline_variant_id,
                ) {
                    PipelineAdmission::Ready(()) => {
                        mesh_pipelines
                            .record_bound_mesh_pass_pipeline(kind, command.pipeline_variant_id);
                        pass.set_pipeline(
                            mesh_pipelines
                                .shadow_pipeline_for_ready_variant(command.pipeline_variant_id),
                        );
                    }
                    PipelineAdmission::Deferred(unavailable)
                    | PipelineAdmission::Failed(unavailable) => {
                        mesh_pipelines.record_pipeline_fallback_for_command_variant(
                            command,
                            command.pipeline_variant_id,
                            SHADOW_ATLAS_PIPELINE_CONSUMER,
                            unavailable,
                        );
                        replayer.invalidate_state_after_external_pipeline();
                        return false;
                    }
                }
            }
            replayer.bind_standard_material_if_needed(pass, command);
            replayer.bind_gpu_scene_if_needed(pass, command, gpu_scene_bind_group);
            replayer.bind_geometry_if_needed(pass, command);
            true
        });
        replayer.stats()
    }
}

fn create_slot_scene_bind_group(
    device: &wgpu::Device,
    scene_layout: &wgpu::BindGroupLayout,
    environment: &ShadowSceneEnvironmentBindingLease,
    uniform_buffer: &wgpu::Buffer,
    uniform_offset: u64,
    uniform_size: NonZeroU64,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-shadow-map-slot-scene-bind-group"),
        layout: scene_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: uniform_buffer,
                    offset: uniform_offset,
                    size: Some(uniform_size),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&environment.black_cube_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&environment.sampler),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&environment.brdf_lut_view),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(&environment.black_cube_view),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(&environment.black_cube_view),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: environment.sh9_buffer.as_entire_binding(),
            },
        ],
    })
}

fn align_up_u64(value: u64, alignment: u64) -> Option<u64> {
    let remainder = value % alignment;
    if remainder == 0 {
        Some(value)
    } else {
        value.checked_add(alignment - remainder)
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
        view_proj_unjittered: view_proj_cols,
        inverse_view_proj: finite_mat4_or_identity(view_proj.inverse()).to_cols_array_2d(),
        ambient_color: [0.0, 0.0, 0.0, 1.0],
        lightmapped_ambient_color: [0.0, 0.0, 0.0, 1.0],
        previous_view_proj_unjittered: view_proj_cols,
        motion_params: [0.0, 0.0, 0.0, 0.0],
        jitter_params: [0.0, 0.0, 0.0, 0.0],
        camera_world_position: [0.0, 0.0, 0.0, 1.0],
        camera_view_direction: [0.0, 0.0, 1.0, 0.0],
        sky_horizon_color: [0.0, 0.0, 0.0, 1.0],
        sky_zenith_color: [0.0, 0.0, 0.0, 1.0],
        sky_ground_color: [0.0, 0.0, 0.0, 1.0],
        sky_sun_direction: [0.0, 0.0, 0.0, 0.0],
        sky_sun_color_radius: [0.0, 0.0, 0.0, 0.0],
        sky_sun_params: [0.0, 0.0, 0.0, 0.0],
        environment_params: [0.0, 0.0, 0.0, 0.0],
        environment_sample_params: [0.0, 0.0, 0.0, 0.0],
        environment_rotation_sin_cos: [0.0, 1.0, 0.0, 0.0],
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
    total.indirect_count_draw_call_count = total
        .indirect_count_draw_call_count
        .saturating_add(next.indirect_count_draw_call_count);
    total.fixed_multi_draw_call_count = total
        .fixed_multi_draw_call_count
        .saturating_add(next.fixed_multi_draw_call_count);
    total.per_draw_indirect_draw_call_count = total
        .per_draw_indirect_draw_call_count
        .saturating_add(next.per_draw_indirect_draw_call_count);
    total.direct_draw_call_count = total
        .direct_draw_call_count
        .saturating_add(next.direct_draw_call_count);
    total.state_change_count = total
        .state_change_count
        .saturating_add(next.state_change_count);
    total.bind_skip_count = total.bind_skip_count.saturating_add(next.bind_skip_count);
    total.material_bind_group_set_count = total
        .material_bind_group_set_count
        .saturating_add(next.material_bind_group_set_count);
    total.material_bind_group_skip_count = total
        .material_bind_group_skip_count
        .saturating_add(next.material_bind_group_skip_count);
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

    use bytemuck::bytes_of;
    use wgpu::util::DeviceExt;

    use crate::core::framework::render::RenderPhase;
    use crate::graphics::backend::RenderBackend;
    use crate::graphics::scene::resources::default_pipeline_key;
    use crate::graphics::scene::scene_renderer::environment::scene_bind_group_layout_entries;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
        DrawInstanceSource, MeshDrawArgs, MeshDrawCommand, MeshGeometryHandle,
        MeshPassPipelineKind, MeshPipelineVariantId,
    };
    use crate::graphics::scene::scene_renderer::primitives::SceneEnvironmentSh9;

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

    #[test]
    fn shadow_view_filter_disables_global_indirect_batches_before_replay() {
        let source = include_str!("shadow_map_renderer.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source section should exist");

        assert!(source.contains("if visible_entities.is_some()"));
        assert!(source.contains("mesh_draw_commands.without_indirect()"));
    }

    #[test]
    fn shadow_atlas_binds_forward_shadow_receiver_layout_slot() {
        let source = include_str!("shadow_map_renderer.rs");

        assert!(source.contains("create_forward_shadow_receiver_bind_group"));
        assert!(
            source.contains("pass.set_bind_group(1, &forward_shadow_receiver_bind_group, &[])")
        );
        assert!(source.contains("replayer.bind_standard_material_if_needed(pass, command);"));
    }

    #[test]
    fn shadow_atlas_uses_one_persistent_aligned_scene_uniform_workspace() {
        let production = include_str!("shadow_map_renderer.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source section should exist");
        let record = production
            .split("fn record_atlas_commands_with_attachment_ops")
            .nth(1)
            .and_then(|source| source.split("fn replay_shadow_command_stream").next())
            .expect("shadow atlas recording must remain bounded");
        let prepare = production
            .split("fn prepare_slot_scene_uploads")
            .nth(1)
            .and_then(|source| source.split("fn record_atlas_commands").next())
            .expect("shadow slot preparation must remain bounded");

        assert!(!record.contains("queue:"));
        assert!(!record.contains("create_buffer_init"));
        assert!(!record.contains("create_bind_group"));
        assert!(prepare.contains("checked_next_power_of_two()"));
        assert!(prepare.contains("min_uniform_buffer_offset_alignment"));
        assert!(prepare.contains("WgpuBufferUpload::new(buffer, 0, payload, 0..payload_len)"));
        assert!(prepare.contains("wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST"));
    }

    #[test]
    fn shadow_environment_bindings_are_leased_without_private_resource_creation() {
        let production = include_str!("shadow_map_renderer.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source section should exist");
        let constructor = production
            .split("impl ShadowMapRenderer")
            .nth(1)
            .and_then(|source| source.split("fn new(").nth(1))
            .and_then(|source| source.split("fn prepare_slot_scene_uploads").next())
            .expect("shadow constructor must remain bounded");
        let slot_bindings = production
            .split("fn create_slot_scene_bind_group")
            .nth(1)
            .and_then(|source| source.split("fn align_up_u64").next())
            .expect("shadow scene binding must remain bounded");

        assert!(!constructor.contains("device:"));
        assert!(!constructor.contains("create_texture"));
        assert!(!constructor.contains("create_sampler"));
        assert!(!constructor.contains("create_buffer"));
        assert_eq!(
            slot_bindings
                .matches("&environment.black_cube_view")
                .count(),
            3
        );
        assert!(slot_bindings.contains("&environment.brdf_lut_view"));
        assert!(slot_bindings.contains("&environment.sampler"));
        assert!(slot_bindings.contains("environment.sh9_buffer.as_entire_binding()"));
    }

    #[test]
    fn shadow_map_scene_bind_group_matches_environment_scene_layout() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let scene_layout_entries = scene_bind_group_layout_entries();
        let scene_layout =
            backend
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("zircon-test-shadow-map-scene-layout"),
                    entries: &scene_layout_entries,
                });

        let error_scope = backend
            .device
            .push_error_scope(wgpu::ErrorFilter::Validation);
        let mut renderer = super::ShadowMapRenderer::new(
            &scene_layout,
            test_shadow_environment_binding_lease(&backend.device),
        );
        renderer
            .ensure_slot_scene_capacity(&backend.device, 1)
            .expect("shadow slot workspace should support one scene uniform");
        let _scene_bind_group = renderer
            .slot_scene_workspace
            .bind_groups
            .first()
            .expect("prepared shadow slot workspace should publish one bind group");
        let error = pollster::block_on(error_scope.pop());

        assert!(
            error.is_none(),
            "shadow-map scene bind group should match scene environment layout: {error:?}"
        );
    }

    #[test]
    fn shadow_scene_uniform_stride_alignment_is_checked() {
        assert_eq!(super::align_up_u64(432, 256), Some(512));
        assert_eq!(super::align_up_u64(512, 256), Some(512));
        assert_eq!(super::align_up_u64(u64::MAX, 256), None);
    }

    fn test_command(source_entity: u64) -> MeshDrawCommand {
        MeshDrawCommand::new(
            RenderPhase::Shadow,
            MeshPassPipelineKind::ShadowDepth,
            default_pipeline_key(),
            MeshPipelineVariantId::new(1),
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

    fn test_shadow_environment_binding_lease(
        device: &wgpu::Device,
    ) -> super::ShadowSceneEnvironmentBindingLease {
        let black_cube_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-test-shadow-black-cube"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let black_cube_view = black_cube_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("zircon-test-shadow-black-cube-view"),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let brdf_lut_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-test-shadow-brdf-lut"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rg16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let brdf_lut_view = brdf_lut_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sh9_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-test-shadow-environment-sh9"),
            contents: bytes_of(&SceneEnvironmentSh9::default()),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        super::ShadowSceneEnvironmentBindingLease::new(
            black_cube_texture,
            black_cube_view,
            sampler,
            brdf_lut_texture,
            brdf_lut_view,
            sh9_buffer,
        )
    }
}
