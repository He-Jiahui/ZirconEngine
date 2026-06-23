use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::core::framework::render::{
    render_mesh_stable_instance_key, PrimitiveRelevance, RenderMeshSnapshot,
};
use crate::core::framework::scene::EntityId;
use crate::core::math::RenderVec4;
use crate::graphics::scene::gpu_scene::{
    GpuInstanceData, GpuPrimitiveData, GpuScene, GpuSceneEntry, GpuSceneUploadReport,
    GPU_PRIMITIVE_FLAG_CAST_SHADOWS, GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM,
    GPU_PRIMITIVE_FLAG_VISIBLE, GPU_SCENE_INVALID_PAYLOAD_SLOT,
};
use crate::graphics::scene::resources::{GpuMeshResource, ResourceStreamer};
use crate::graphics::scene::scene_renderer::lighting::light_buffer::pack_lighting_extract;
use crate::graphics::scene::scene_renderer::shadow::ShadowLightSlotAssignments;
use crate::graphics::types::ViewportRenderFrame;

use super::super::super::super::primitives::render_vec4_or;
use super::super::super::mesh_draw::VirtualGeometrySubmissionDetail;
use super::super::super::mesh_draw::{
    MaterialTextureSet, MeshDraw, MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
};
use super::super::super::mesh_pass::MeshPassCommandBuffers;
use super::super::super::prepared_queue::{
    summarize_prepared_mesh_queue_items, PreparedMeshQueueStats,
};
use super::super::create_mesh_draw::create_mesh_draw;
use super::super::indexed_indirect_args::IndexedIndirectArgs;
use super::build_mesh_draw_build_context::build_mesh_draw_build_context;
use super::extend_pending_draws_for_mesh_instance::extend_pending_draws_for_mesh_instance;
use super::pending_command_cache_extract::{
    extract_pending_static_mesh_command_cache_hits, PendingMeshCommandCacheExtractionContext,
    PendingMeshCommandCacheExtractionStats,
};
use super::pending_command_cache_plan::{
    summarize_pending_mesh_command_cache_plan, PendingMeshCommandCachePlanStats,
    PendingMeshCommandCacheVisibility,
};
use super::pending_mesh_draw::{PendingMeshGeometry, PendingSkinnedGpuSource};
use super::phase_ordering::phase_ordered_meshes;
use super::previous_skinned_palette::{
    previous_skinned_gpu_state_for_gpu_scene_entry, skinned_gpu_source_state_for_pending_draw,
    skinned_joint_palette_state_for_pending_draw,
};
use super::virtual_geometry_indirect::build_virtual_geometry_indirect_draw_plan;

pub(crate) struct BuiltMeshDraws {
    draws: Vec<MeshDraw>,
    prepared_mesh_queue_stats: PreparedMeshQueueStats,
    prebuilt_mesh_pass_command_buffers: MeshPassCommandBuffers,
    gpu_scene_upload_report: GpuSceneUploadReport,
    indirect_segment_count: u32,
    indirect_args_count: u32,
    indirect_args_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    indirect_submission_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    indirect_authority_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    indirect_draw_ref_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    indirect_segment_buffer: Option<std::sync::Arc<wgpu::Buffer>>,
    pending_command_cache_plan_stats: PendingMeshCommandCachePlanStats,
    pending_command_cache_extraction_stats: PendingMeshCommandCacheExtractionStats,
}

impl BuiltMeshDraws {
    pub(crate) fn into_draws(self) -> Vec<MeshDraw> {
        self.draws
    }

    pub(crate) fn prepared_mesh_queue_stats(&self) -> PreparedMeshQueueStats {
        self.prepared_mesh_queue_stats
    }

    pub(crate) fn prebuilt_mesh_pass_command_buffers(&self) -> MeshPassCommandBuffers {
        self.prebuilt_mesh_pass_command_buffers.clone()
    }

    pub(crate) fn gpu_scene_upload_report(&self) -> GpuSceneUploadReport {
        self.gpu_scene_upload_report
    }

    pub(crate) fn indirect_segment_count(&self) -> u32 {
        self.indirect_segment_count
    }

    pub(crate) fn indirect_args_count(&self) -> u32 {
        self.indirect_args_count
    }

    pub(crate) fn indirect_args_buffer(&self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_args_buffer.clone()
    }

    pub(crate) fn indirect_submission_buffer(&self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_submission_buffer.clone()
    }

    pub(crate) fn indirect_authority_buffer(&self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_authority_buffer.clone()
    }

    pub(crate) fn indirect_draw_ref_buffer(&self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_draw_ref_buffer.clone()
    }

    pub(crate) fn indirect_segment_buffer(&self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_segment_buffer.clone()
    }

    pub(crate) fn pending_command_cache_plan_stats(&self) -> PendingMeshCommandCachePlanStats {
        self.pending_command_cache_plan_stats
    }

    pub(crate) fn pending_command_cache_extraction_stats(
        &self,
    ) -> PendingMeshCommandCacheExtractionStats {
        self.pending_command_cache_extraction_stats
    }
}

pub(crate) fn build_mesh_draws(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    _encoder: &mut wgpu::CommandEncoder,
    material_texture_layout: &wgpu::BindGroupLayout,
    gpu_scene: &mut GpuScene,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    virtual_geometry_enabled: bool,
    shadow_light_slots: Option<&ShadowLightSlotAssignments>,
    command_cache_extraction: Option<PendingMeshCommandCacheExtractionContext<'_>>,
) -> BuiltMeshDraws {
    let build_context = build_mesh_draw_build_context(frame, virtual_geometry_enabled);
    let mut pending_draws = Vec::new();
    for mesh_instance in phase_ordered_meshes(frame, streamer) {
        extend_pending_draws_for_mesh_instance(
            &mut pending_draws,
            streamer,
            frame,
            &build_context,
            mesh_instance.snapshot,
            mesh_instance.command_sort_input,
        );
    }
    let indirect_plan = build_virtual_geometry_indirect_draw_plan(
        device,
        frame,
        virtual_geometry_enabled,
        &mut pending_draws,
    );
    let mut packed_lights =
        pack_lighting_extract(&frame.extract.lighting, frame.preview().lighting_enabled);
    if let Some(shadow_light_slots) = shadow_light_slots {
        shadow_light_slots
            .apply_to_packed_lights(&frame.extract.lighting, &mut packed_lights.lights);
    }
    gpu_scene.write_lights(device, &packed_lights.lights);
    let (gpu_scene_upload_report, gpu_scene_entries) =
        sync_gpu_scene_pending_draws(device, queue, gpu_scene, &mut pending_draws);
    let visibility_states = mesh_visibility_states(frame);
    let pending_command_cache_plan_stats =
        summarize_pending_mesh_command_cache_plan(&pending_draws, |entity| {
            visibility_states
                .get(&entity)
                .copied()
                .map(PendingMeshCommandCacheVisibility::from)
        });
    let prepared_mesh_queue_stats =
        prepared_mesh_queue_stats_for_pending_draws(&pending_draws, &gpu_scene_entries);
    let indirect_segment_count = indirect_plan.segment_count;
    let indirect_args_count = indirect_plan.args_count;
    let indirect_draw_ref_buffer = indirect_plan.draw_ref_buffer;
    let indirect_submission_buffer = indirect_plan.submission_buffer;
    let indirect_authority_buffer = indirect_plan.authority_buffer;
    let indirect_segment_buffer = indirect_plan.segment_buffer;
    let indirect_args_offsets = indirect_plan.args_offsets;
    let indirect_args_buffers = indirect_plan.args_buffers;
    let pending_draw_draw_ref_indices = indirect_plan.draw_ref_indices;
    let pending_draw_submission_tokens = indirect_plan.submission_tokens;
    let pending_draw_submission_details = indirect_plan.submission_details;
    let indirect_args_buffer = indirect_plan.args_buffer;
    let indirect_args_stride = std::mem::size_of::<IndexedIndirectArgs>() as u64;

    let pending_draws = pending_draws.into_iter().map(Some).collect::<Vec<_>>();
    let (pending_draws, prebuilt_mesh_pass_command_buffers, pending_command_cache_extraction_stats) =
        if let Some(command_cache_extraction) = command_cache_extraction {
            let extraction = extract_pending_static_mesh_command_cache_hits(
                pending_draws,
                |entity| {
                    visibility_states
                        .get(&entity)
                        .copied()
                        .map(PendingMeshCommandCacheVisibility::from)
                },
                |entity, draw_ordinal| {
                    gpu_scene_entries
                        .get(&render_mesh_stable_instance_key(entity, draw_ordinal))
                        .map(|entry| (entry.entry.first_instance_index, entry.entry.instance_count))
                },
                command_cache_extraction,
            );
            (
                extraction.pending_draws,
                extraction.command_buffers,
                extraction.stats,
            )
        } else {
            (
                pending_draws,
                MeshPassCommandBuffers::default(),
                PendingMeshCommandCacheExtractionStats::default(),
            )
        };
    let mut ordered_pending_draws = Vec::new();
    ordered_pending_draws.extend(pending_draws.into_iter().enumerate().filter_map(
        |(index, pending_draw)| {
            let pending_draw = pending_draw?;
            let indirect_args_offset = indirect_args_offsets
                .get(index)
                .copied()
                .flatten()
                .unwrap_or((index as u64) * indirect_args_stride);
            Some((
                indirect_args_offset,
                indirect_args_buffers.get(index).cloned().flatten(),
                index,
                pending_draw_submission_details
                    .get(index)
                    .copied()
                    .flatten()
                    .or_else(|| {
                        submission_detail_from_draw_ref(
                            pending_draw.indirect_draw_ref,
                            pending_draw_submission_tokens.get(index).copied().flatten(),
                            pending_draw_draw_ref_indices.get(index).copied().flatten(),
                            Some(indirect_args_offset),
                            indirect_args_stride,
                        )
                    }),
                pending_draw,
            ))
        },
    ));
    BuiltMeshDraws {
        draws: ordered_pending_draws
            .into_iter()
            .map(
                |(
                    indirect_args_offset,
                    draw_indirect_args_buffer,
                    original_index,
                    submission_detail,
                    pending_draw,
                )| {
                    let synced_gpu_scene_entry = gpu_scene_entries
                        .get(&render_mesh_stable_instance_key(
                            pending_draw.source_entity,
                            pending_draw.source_draw_ordinal,
                        ))
                        .expect("pending mesh draw must have a synchronized GPUScene entry");
                    let gpu_scene_instance_span = (
                        synced_gpu_scene_entry.entry.first_instance_index,
                        synced_gpu_scene_entry.entry.instance_count,
                    );
                    let supports_skinned_gpu_skinning =
                        pending_draw.pipeline_key.uses_fallback_shader();
                    let skinned_gpu_source = pending_draw.skinned_gpu_source;
                    let resolved_skinned_gpu_source =
                        supports_skinned_gpu_skinning.then_some(()).and_then(|_| {
                            pending_draw
                                .resolved_skinned_gpu_source
                                .as_ref()
                                .map(|mesh| {
                                    (
                                        mesh.clone(),
                                        skinned_gpu_source
                                            .as_ref()
                                            .map(skinned_gpu_source_geometry_source)
                                            .expect(
                                                "resolved skinned GPU source has source metadata",
                                            ),
                                        skinned_gpu_source.as_ref().is_some_and(
                                            PendingSkinnedGpuSource::uses_cpu_morphed_source,
                                        ),
                                    )
                                })
                        });
                    let (
                        mesh,
                        geometry_source,
                        resolved_skinned_gpu_source,
                        skinned_gpu_source_uses_cpu_morphed_source,
                        skinned_gpu_skinning_enabled,
                    ) = match pending_draw.mesh {
                        PendingMeshGeometry::Prepared(mesh) => {
                            (mesh, MeshDrawGeometrySource::Prepared, None, false, false)
                        }
                        PendingMeshGeometry::Dynamic(primitive) => {
                            if let Some((mesh, geometry_source, uses_cpu_morphed_source)) =
                                resolved_skinned_gpu_source
                            {
                                (
                                    mesh.clone(),
                                    geometry_source,
                                    Some(mesh),
                                    uses_cpu_morphed_source,
                                    true,
                                )
                            } else {
                                (
                                    std::sync::Arc::new(GpuMeshResource::from_asset(
                                        device, primitive,
                                    )),
                                    MeshDrawGeometrySource::Dynamic,
                                    None,
                                    false,
                                    false,
                                )
                            }
                        }
                    };
                    let previous_skinned_joint_palette = if skinned_gpu_skinning_enabled {
                        pending_draw.previous_skinned_joint_palette
                    } else {
                        None
                    };
                    let has_previous_velocity_transform = synced_gpu_scene_entry
                        .has_previous_velocity_transform
                        && (!skinned_gpu_skinning_enabled
                            || previous_skinned_joint_palette.is_some());
                    let mut mesh_draw = create_mesh_draw(
                        device,
                        gpu_scene,
                        material_texture_layout,
                        mesh,
                        geometry_source,
                        pending_draw.mobility,
                        pending_draw.source_entity,
                        pending_draw.source_draw_ordinal,
                        pending_draw.static_state,
                        pending_draw.material_textures,
                        pending_draw.material_uniform,
                        pending_draw.standard_material_uniform,
                        pending_draw.pipeline_key,
                        pending_draw.cast_shadows,
                        pending_draw.taa_reactive_mask_strength,
                        has_previous_velocity_transform,
                        pending_draw.mesh_lod,
                        pending_draw.skinned,
                        pending_draw.skinned_joint_palette,
                        previous_skinned_joint_palette,
                        pending_draw.previous_skinned_gpu_source,
                        resolved_skinned_gpu_source,
                        skinned_gpu_source_uses_cpu_morphed_source,
                        skinned_gpu_skinning_enabled,
                        pending_draw.first_index,
                        pending_draw.draw_index_count,
                        draw_indirect_args_buffer,
                        indirect_args_offset,
                        submission_detail.or_else(|| {
                            submission_detail_from_draw_ref(
                                pending_draw.indirect_draw_ref,
                                pending_draw_submission_tokens
                                    .get(original_index)
                                    .copied()
                                    .flatten(),
                                pending_draw_draw_ref_indices
                                    .get(original_index)
                                    .copied()
                                    .flatten(),
                                Some(indirect_args_offset),
                                indirect_args_stride,
                            )
                        }),
                    )
                    .with_gpu_scene_instance_span(
                        gpu_scene_instance_span.0,
                        gpu_scene_instance_span.1,
                    )
                    .with_command_sort_input(pending_draw.command_sort_input);
                    if let Some(visibility) =
                        visibility_states.get(&pending_draw.source_entity).copied()
                    {
                        mesh_draw = mesh_draw.with_visibility(
                            visibility.relevance,
                            visibility.main_view_visible,
                            visibility.shadow_view_visible,
                        );
                    }
                    mesh_draw
                },
            )
            .collect(),
        prepared_mesh_queue_stats,
        prebuilt_mesh_pass_command_buffers,
        gpu_scene_upload_report,
        indirect_segment_count,
        indirect_args_count,
        indirect_args_buffer,
        indirect_submission_buffer,
        indirect_authority_buffer,
        indirect_draw_ref_buffer,
        indirect_segment_buffer,
        pending_command_cache_plan_stats,
        pending_command_cache_extraction_stats,
    }
}

#[derive(Clone, Copy)]
struct MeshVisibilityState {
    relevance: PrimitiveRelevance,
    main_view_visible: bool,
    shadow_view_visible: bool,
}

impl From<MeshVisibilityState> for PendingMeshCommandCacheVisibility {
    fn from(visibility: MeshVisibilityState) -> Self {
        Self::new(
            visibility.relevance,
            visibility.main_view_visible,
            visibility.shadow_view_visible,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct PendingPreparedMeshBatchKey {
    geometry_source: MeshDrawGeometrySource,
    mesh: usize,
    base_color_texture: usize,
    normal_texture: usize,
    metallic_roughness_texture: usize,
    occlusion_texture: usize,
    emissive_texture: usize,
    material_uniform: usize,
    standard_material_uniform: usize,
    pipeline_key: crate::graphics::scene::resources::PipelineKey,
    first_index: u32,
    draw_index_count: u32,
}

fn prepared_mesh_queue_stats_for_pending_draws(
    pending_draws: &[super::pending_mesh_draw::PendingMeshDraw],
    gpu_scene_entries: &HashMap<u64, SyncedGpuSceneEntry>,
) -> PreparedMeshQueueStats {
    summarize_prepared_mesh_queue_items(pending_draws.iter().map(|pending_draw| {
        let skinned_gpu_skinning_enabled = pending_draw.pipeline_key.uses_fallback_shader()
            && pending_draw.resolved_skinned_gpu_source.is_some();
        let has_previous_velocity_transform = gpu_scene_entries
            .get(&render_mesh_stable_instance_key(
                pending_draw.source_entity,
                pending_draw.source_draw_ordinal,
            ))
            .map(|entry| {
                entry.has_previous_velocity_transform
                    && (!skinned_gpu_skinning_enabled
                        || pending_draw.previous_skinned_joint_palette.is_some())
            })
            .unwrap_or(false);
        let geometry_source =
            pending_mesh_draw_geometry_source(pending_draw, skinned_gpu_skinning_enabled);
        let queue_profile = MeshDrawQueueProfile::new(
            MeshDrawQueuePhase::from_pipeline_flags(
                pending_draw.pipeline_key.is_transparent(),
                pending_draw.pipeline_key.is_alpha_mask(),
            ),
            geometry_source,
            pending_draw.mobility,
            false,
            skinned_gpu_skinning_enabled,
            pending_draw.mesh_lod.is_some(),
        );
        (
            queue_profile,
            pending_draw.cast_shadows && queue_profile.phase().casts_shadow(),
            has_previous_velocity_transform,
            pending_draw.skinned,
            pending_draw.skinned_joint_palette.is_some(),
            skinned_gpu_skinning_enabled && pending_draw.previous_skinned_joint_palette.is_some(),
            pending_draw.skinned_gpu_source.is_some(),
            pending_draw
                .skinned_gpu_source
                .as_ref()
                .is_some_and(PendingSkinnedGpuSource::uses_cpu_morphed_source),
            skinned_gpu_skinning_enabled,
            pending_mesh_draw_batch_key(pending_draw, geometry_source),
        )
    }))
}

fn pending_mesh_draw_geometry_source(
    pending_draw: &super::pending_mesh_draw::PendingMeshDraw,
    skinned_gpu_skinning_enabled: bool,
) -> MeshDrawGeometrySource {
    match &pending_draw.mesh {
        PendingMeshGeometry::Prepared(_) => MeshDrawGeometrySource::Prepared,
        PendingMeshGeometry::Dynamic(_) if skinned_gpu_skinning_enabled => pending_draw
            .skinned_gpu_source
            .as_ref()
            .map(skinned_gpu_source_geometry_source)
            .unwrap_or(MeshDrawGeometrySource::Dynamic),
        PendingMeshGeometry::Dynamic(_) => MeshDrawGeometrySource::Dynamic,
    }
}

fn pending_mesh_draw_batch_key(
    pending_draw: &super::pending_mesh_draw::PendingMeshDraw,
    geometry_source: MeshDrawGeometrySource,
) -> PendingPreparedMeshBatchKey {
    PendingPreparedMeshBatchKey {
        geometry_source,
        mesh: pending_mesh_identity(pending_draw),
        base_color_texture: material_texture_identity(
            &pending_draw.material_textures,
            |textures| &textures.base_color,
        ),
        normal_texture: material_texture_identity(&pending_draw.material_textures, |textures| {
            &textures.normal
        }),
        metallic_roughness_texture: material_texture_identity(
            &pending_draw.material_textures,
            |textures| &textures.metallic_roughness,
        ),
        occlusion_texture: material_texture_identity(&pending_draw.material_textures, |textures| {
            &textures.occlusion
        }),
        emissive_texture: material_texture_identity(&pending_draw.material_textures, |textures| {
            &textures.emissive
        }),
        material_uniform: Arc::as_ptr(&pending_draw.material_uniform) as usize,
        standard_material_uniform: Arc::as_ptr(&pending_draw.standard_material_uniform) as usize,
        pipeline_key: pending_draw.pipeline_key.clone(),
        first_index: pending_draw.first_index,
        draw_index_count: pending_draw.draw_index_count,
    }
}

fn pending_mesh_identity(pending_draw: &super::pending_mesh_draw::PendingMeshDraw) -> usize {
    if let Some(mesh) = pending_draw.resolved_skinned_gpu_source.as_ref() {
        return Arc::as_ptr(mesh) as usize;
    }
    match &pending_draw.mesh {
        PendingMeshGeometry::Prepared(mesh) => Arc::as_ptr(mesh) as usize,
        PendingMeshGeometry::Dynamic(_) => {
            (pending_draw.source_entity as usize).wrapping_mul(31)
                ^ pending_draw.source_draw_ordinal as usize
        }
    }
}

fn material_texture_identity(
    textures: &MaterialTextureSet,
    select: impl FnOnce(&MaterialTextureSet) -> &super::super::super::mesh_draw::MaterialTextureBinding,
) -> usize {
    select(textures).identity()
}

fn mesh_visibility_states(frame: &ViewportRenderFrame) -> HashMap<EntityId, MeshVisibilityState> {
    let Some(frame_visibility) = frame.frame_visibility() else {
        return HashMap::new();
    };
    let main_visible_indices = frame_visibility
        .main_view()
        .map(|view| view.visible.iter().copied().collect::<HashSet<_>>())
        .unwrap_or_default();
    let shadow_views = frame_visibility.shadow_views().collect::<Vec<_>>();
    let shadow_visible_indices = shadow_views
        .iter()
        .flat_map(|view| view.visible.iter().copied())
        .collect::<HashSet<_>>();
    let has_shadow_views = !shadow_views.is_empty();

    frame_visibility
        .entities
        .iter()
        .enumerate()
        .map(|(index, entity)| {
            let index =
                u32::try_from(index).expect("frame visibility primitive index exceeds u32 range");
            let relevance = frame_visibility
                .relevance
                .get(index as usize)
                .copied()
                .unwrap_or_default();
            (
                *entity,
                MeshVisibilityState {
                    relevance,
                    main_view_visible: main_visible_indices.contains(&index),
                    shadow_view_visible: if has_shadow_views {
                        shadow_visible_indices.contains(&index)
                    } else {
                        relevance.shadow_caster()
                    },
                },
            )
        })
        .collect()
}

#[derive(Clone, Copy)]
struct SyncedGpuSceneEntry {
    entry: GpuSceneEntry,
    has_previous_velocity_transform: bool,
}

fn sync_gpu_scene_pending_draws(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    gpu_scene: &mut GpuScene,
    pending_draws: &mut [super::pending_mesh_draw::PendingMeshDraw],
) -> (GpuSceneUploadReport, HashMap<u64, SyncedGpuSceneEntry>) {
    let mut live_keys = HashSet::new();
    let mut entries = HashMap::new();
    for pending_draw in pending_draws {
        let stable_instance_key = render_mesh_stable_instance_key(
            pending_draw.source_entity,
            pending_draw.source_draw_ordinal,
        );
        live_keys.insert(stable_instance_key);
        let entry = gpu_scene.register(device, stable_instance_key, 1);
        pending_draw.resolved_skinned_gpu_source =
            resolved_skinned_gpu_source_for_pending_draw(device, pending_draw);
        let previous_skinned_gpu_state = previous_skinned_gpu_state_for_gpu_scene_entry(
            gpu_scene,
            stable_instance_key,
            pending_draw,
        );
        pending_draw.previous_skinned_joint_palette = previous_skinned_gpu_state.joint_palette;
        pending_draw.previous_skinned_gpu_source = previous_skinned_gpu_state.source;
        gpu_scene.stage_current_skinned_joint_palette(
            stable_instance_key,
            skinned_joint_palette_state_for_pending_draw(pending_draw),
        );
        gpu_scene.stage_current_skinned_gpu_source(
            stable_instance_key,
            skinned_gpu_source_state_for_pending_draw(pending_draw),
        );
        let (previous_model_matrix, has_previous_velocity_transform) =
            previous_model_matrix_for_gpu_scene_entry(gpu_scene, pending_draw, entry);
        gpu_scene.write_primitive(
            entry,
            primitive_data_for_pending_draw(pending_draw, entry, has_previous_velocity_transform),
        );
        gpu_scene.write_instances(
            entry,
            &[instance_data_for_pending_draw(
                pending_draw,
                entry,
                previous_model_matrix,
            )],
        );
        gpu_scene.set_transform_revision(stable_instance_key, pending_draw.transform_revision);
        entries.insert(
            stable_instance_key,
            SyncedGpuSceneEntry {
                entry,
                has_previous_velocity_transform,
            },
        );
    }
    gpu_scene.retain_registered_keys(&live_keys);
    (gpu_scene.flush_updates(queue), entries)
}

fn primitive_data_for_pending_draw(
    pending_draw: &super::pending_mesh_draw::PendingMeshDraw,
    entry: GpuSceneEntry,
    has_previous_velocity_transform: bool,
) -> GpuPrimitiveData {
    let mut flags = GPU_PRIMITIVE_FLAG_VISIBLE;
    if pending_draw.cast_shadows {
        flags |= GPU_PRIMITIVE_FLAG_CAST_SHADOWS;
    }
    if has_previous_velocity_transform {
        flags |= GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM;
    }

    GpuPrimitiveData {
        bounds_center: [
            pending_draw.model_matrix[3][0],
            pending_draw.model_matrix[3][1],
            pending_draw.model_matrix[3][2],
        ],
        bounds_radius: approximate_transform_radius(&pending_draw.model_matrix),
        tint: render_vec4_or(pending_draw.draw_tint, RenderVec4::ONE).to_array(),
        shadow_params: shadow_params_from_pending_draw(pending_draw),
        motion_params: motion_params_from_pending_draw(
            pending_draw,
            has_previous_velocity_transform,
        ),
        flags,
        first_instance_index: entry.first_instance_index,
        instance_count: entry.instance_count,
        payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
    }
}

fn instance_data_for_pending_draw(
    pending_draw: &super::pending_mesh_draw::PendingMeshDraw,
    entry: GpuSceneEntry,
    previous_model_matrix: [[f32; 4]; 4],
) -> GpuInstanceData {
    GpuInstanceData {
        world_from_local: pending_draw.model_matrix,
        prev_world_from_local: previous_model_matrix,
        primitive_index: entry.primitive_index,
        flags: 0,
        payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
        _pad0: 0,
    }
}

fn previous_model_matrix_for_gpu_scene_entry(
    gpu_scene: &GpuScene,
    pending_draw: &super::pending_mesh_draw::PendingMeshDraw,
    entry: GpuSceneEntry,
) -> ([[f32; 4]; 4], bool) {
    gpu_scene
        .previous_world_from_local(entry)
        .map(|previous| (previous, true))
        .unwrap_or((pending_draw.model_matrix, false))
}

fn shadow_params_from_pending_draw(
    pending_draw: &super::pending_mesh_draw::PendingMeshDraw,
) -> [f32; 4] {
    let alpha_cutoff = pending_draw
        .pipeline_key
        .alpha_cutoff_bits
        .map(f32::from_bits)
        .filter(|cutoff| cutoff.is_finite())
        .unwrap_or(0.5)
        .clamp(0.0, 1.0);

    [
        if pending_draw.pipeline_key.is_alpha_mask() {
            1.0
        } else {
            0.0
        },
        alpha_cutoff,
        if pending_draw.receive_shadows {
            1.0
        } else {
            0.0
        },
        0.0,
    ]
}

fn motion_params_from_pending_draw(
    pending_draw: &super::pending_mesh_draw::PendingMeshDraw,
    has_previous_velocity_transform: bool,
) -> [f32; 4] {
    [
        if has_previous_velocity_transform {
            1.0
        } else {
            0.0
        },
        if pending_draw.skinned { 1.0 } else { 0.0 },
        if pending_draw.skinned
            && has_previous_velocity_transform
            && pending_draw.previous_skinned_joint_palette.is_some()
        {
            1.0
        } else {
            0.0
        },
        if pending_draw.pipeline_key.has_normal_texture {
            1.0
        } else {
            0.0
        },
    ]
}

fn approximate_transform_radius(model_matrix: &[[f32; 4]; 4]) -> f32 {
    let x = column_length(model_matrix[0]);
    let y = column_length(model_matrix[1]);
    let z = column_length(model_matrix[2]);
    x.max(y).max(z)
}

fn column_length(column: [f32; 4]) -> f32 {
    (column[0] * column[0] + column[1] * column[1] + column[2] * column[2]).sqrt()
}

fn resolved_skinned_gpu_source_for_pending_draw(
    device: &wgpu::Device,
    pending_draw: &super::pending_mesh_draw::PendingMeshDraw,
) -> Option<std::sync::Arc<GpuMeshResource>> {
    pending_draw
        .pipeline_key
        .uses_fallback_shader()
        .then_some(())?;
    Some(resolve_skinned_gpu_source_mesh(
        device,
        pending_draw.skinned_gpu_source.as_ref()?,
    ))
}

fn resolve_skinned_gpu_source_mesh(
    device: &wgpu::Device,
    source: &PendingSkinnedGpuSource,
) -> std::sync::Arc<GpuMeshResource> {
    match source {
        PendingSkinnedGpuSource::Prepared(mesh) => mesh.clone(),
        PendingSkinnedGpuSource::CpuMorphed { primitive, .. } => {
            std::sync::Arc::new(GpuMeshResource::from_asset(device, primitive.clone()))
        }
    }
}

fn skinned_gpu_source_geometry_source(source: &PendingSkinnedGpuSource) -> MeshDrawGeometrySource {
    match source {
        PendingSkinnedGpuSource::Prepared(_) => MeshDrawGeometrySource::Prepared,
        PendingSkinnedGpuSource::CpuMorphed { .. } => {
            MeshDrawGeometrySource::DynamicGpuSkinningSource
        }
    }
}

fn submission_detail_from_draw_ref(
    draw_ref: Option<super::pending_mesh_draw::VirtualGeometryIndirectDrawRef>,
    submission_token: Option<u32>,
    draw_ref_index: Option<u32>,
    indirect_args_offset: Option<u64>,
    indirect_args_stride: u64,
) -> Option<VirtualGeometrySubmissionDetail> {
    let draw_ref = draw_ref?;
    let submission_token = submission_token.unwrap_or(u32::MAX);
    Some(VirtualGeometrySubmissionDetail::new(
        draw_ref.segment_key.instance_index,
        draw_ref.segment_key.entity,
        draw_ref.segment_key.page_id,
        if submission_token == u32::MAX {
            0
        } else {
            submission_token >> 16
        },
        if submission_token == u32::MAX {
            0
        } else {
            submission_token & 0xffff
        },
        draw_ref_index.unwrap_or_else(|| {
            indirect_args_offset
                .map(|offset| (offset / indirect_args_stride) as u32)
                .unwrap_or_default()
        }),
        draw_ref.segment_key.cluster_start_ordinal,
        draw_ref.segment_key.cluster_span_count,
        draw_ref.segment_key.cluster_total_count,
        draw_ref.segment_key.submission_slot,
        match draw_ref.segment_key.state {
            0 => crate::core::framework::render::RenderVirtualGeometryExecutionState::Resident,
            1 => crate::core::framework::render::RenderVirtualGeometryExecutionState::PendingUpload,
            _ => crate::core::framework::render::RenderVirtualGeometryExecutionState::Missing,
        },
        draw_ref.segment_key.lineage_depth,
        draw_ref.segment_key.lod_level,
        draw_ref.segment_key.frontier_rank,
    ))
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        CorePipelineKind, FallbackSkyboxKind, PreviewEnvironmentExtract, PrimitiveRelevance,
        RenderFrameExtract, RenderLayerSet, RenderMaterialAlphaMode, RenderOverlayExtract,
        RenderSceneGeometryExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
        ViewportCameraSnapshot,
    };
    use crate::core::framework::scene::Mobility;
    use crate::core::math::{UVec2, Vec4};
    use crate::graphics::visibility::{
        FrameVisibility, ViewCullingStats, ViewVisibilityContext, VisibilityBounds,
        VisibilityViewKey,
    };
    use crate::graphics::ViewportRenderFrame;

    #[test]
    fn mesh_visibility_states_preserve_shadow_only_casters() {
        let frame = ViewportRenderFrame::from_extract(
            RenderFrameExtract::from_snapshot(
                RenderWorldSnapshotHandle::new(11),
                RenderSceneSnapshot {
                    scene: RenderSceneGeometryExtract {
                        camera: ViewportCameraSnapshot::default(),
                        meshes: Vec::new(),
                        directional_lights: Vec::new(),
                        point_lights: Vec::new(),
                        spot_lights: Vec::new(),
                        ambient_lights: Vec::new(),
                        rect_lights: Vec::new(),
                    },
                    overlays: RenderOverlayExtract::default(),
                    preview: PreviewEnvironmentExtract {
                        lighting_enabled: true,
                        skybox_enabled: false,
                        fallback_skybox: FallbackSkyboxKind::None,
                        clear_color: Vec4::ZERO,
                    },
                    virtual_geometry_debug: None,
                },
            ),
            UVec2::new(320, 240),
        )
        .with_frame_visibility(FrameVisibility {
            entities: vec![1, 2],
            bounds: vec![
                VisibilityBounds {
                    center: crate::core::math::Vec3::new(0.0, 0.0, -5.0),
                    radius: 1.0,
                },
                VisibilityBounds {
                    center: crate::core::math::Vec3::new(0.0, 8.0, -5.0),
                    radius: 1.0,
                },
            ],
            render_layer_masks: vec![
                RenderLayerSet::from_legacy_mask(u32::MAX),
                RenderLayerSet::from_legacy_mask(u32::MAX),
            ],
            relevance: vec![opaque_shadow_relevance(), opaque_shadow_relevance()],
            relevance_generation: 0,
            views: vec![
                ViewVisibilityContext {
                    view: VisibilityViewKey::MainCamera,
                    camera: ViewportCameraSnapshot::default(),
                    visible: vec![0],
                    stats: ViewCullingStats::default(),
                },
                ViewVisibilityContext {
                    view: VisibilityViewKey::ShadowCascade {
                        light: 99,
                        cascade: 0,
                    },
                    camera: ViewportCameraSnapshot::default(),
                    visible: vec![1],
                    stats: ViewCullingStats::default(),
                },
            ],
        });

        let states = super::mesh_visibility_states(&frame);

        let main_receiver = states.get(&1).expect("main-view receiver state");
        assert!(main_receiver.main_view_visible);
        assert!(!main_receiver.shadow_view_visible);

        let shadow_only_caster = states.get(&2).expect("shadow-only caster state");
        assert!(!shadow_only_caster.main_view_visible);
        assert!(shadow_only_caster.shadow_view_visible);
        assert!(shadow_only_caster.relevance.shadow_caster());
    }

    fn opaque_shadow_relevance() -> PrimitiveRelevance {
        PrimitiveRelevance::for_mesh_view(
            &RenderLayerSet::layer(0),
            CorePipelineKind::Core3d,
            &RenderLayerSet::layer(0),
            Mobility::Static,
            RenderMaterialAlphaMode::Opaque,
        )
    }
}
