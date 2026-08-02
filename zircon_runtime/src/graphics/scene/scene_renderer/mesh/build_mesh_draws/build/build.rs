use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::core::framework::render::PrimitiveRelevance;
use crate::core::framework::scene::EntityId;
use crate::graphics::scene::gpu_scene::{GpuScene, GpuSceneUploadReport};
use crate::graphics::scene::resources::{
    GpuMaterialUniformResource, GpuMeshResource, MaterialDisabledPasses, ResourceStreamer,
};
use crate::graphics::scene::scene_renderer::lighting::light_buffer::pack_lighting_extract_with_cookies;
use crate::graphics::scene::scene_renderer::shadow::ShadowLightSlotAssignments;
use crate::graphics::types::ViewportRenderFrame;

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
use super::geometry_source_selection::{
    pending_draw_has_enabled_skinned_gpu_source, pending_mesh_draw_geometry_source,
    pending_mesh_source_selection, PendingMeshSourceSelection,
};
use super::gpu_scene_sync::{sync_gpu_scene_pending_draws, SyncedGpuSceneEntry};
use super::morph_payload_upload::upload_morph_payloads;
use super::pending_command_cache_extract::{
    extract_pending_static_mesh_command_cache_hits, PendingMeshCommandCacheExtractionContext,
    PendingMeshCommandCacheExtractionStats, PendingMeshDrawRemainder,
};
use super::pending_command_cache_plan::{
    summarize_pending_mesh_command_cache_plan, PendingMeshCommandCachePlanStats,
    PendingMeshCommandCacheVisibility,
};
use super::pending_mesh_draw::{PendingMeshGeometry, PendingSkinnedGpuSource};
use super::phase_ordering::phase_ordered_meshes;
use super::virtual_geometry_indirect::build_virtual_geometry_indirect_draw_plan;
use super::virtual_geometry_resident_upload::upload_virtual_geometry_resident_payloads;

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

    pub(crate) fn prebuilt_mesh_pass_command_buffers(&mut self) -> MeshPassCommandBuffers {
        std::mem::take(&mut self.prebuilt_mesh_pass_command_buffers)
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

    pub(crate) fn indirect_args_buffer(&mut self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_args_buffer.take()
    }

    pub(crate) fn indirect_submission_buffer(&mut self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_submission_buffer.take()
    }

    pub(crate) fn indirect_authority_buffer(&mut self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_authority_buffer.take()
    }

    pub(crate) fn indirect_draw_ref_buffer(&mut self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_draw_ref_buffer.take()
    }

    pub(crate) fn indirect_segment_buffer(&mut self) -> Option<std::sync::Arc<wgpu::Buffer>> {
        self.indirect_segment_buffer.take()
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
    encoder: &mut wgpu::CommandEncoder,
    material_texture_layout: &wgpu::BindGroupLayout,
    gpu_scene: &mut GpuScene,
    streamer: &ResourceStreamer,
    frame: &ViewportRenderFrame,
    virtual_geometry_enabled: bool,
    volumetric_fog_enabled: bool,
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
            gpu_scene,
            mesh_instance.snapshot,
            mesh_instance.command_sort_input,
        );
    }
    for pending_draw in &mut pending_draws {
        pending_draw.pipeline_key.volumetric_fog = volumetric_fog_enabled;
    }
    let indirect_plan = build_virtual_geometry_indirect_draw_plan(
        device,
        frame,
        virtual_geometry_enabled,
        &mut pending_draws,
    );
    let virtual_geometry_upload_report = upload_virtual_geometry_resident_payloads(
        device,
        queue,
        gpu_scene,
        virtual_geometry_enabled,
        frame.virtual_geometry_debug_snapshot.as_ref(),
    );
    let morph_upload_report = upload_morph_payloads(device, queue, gpu_scene, &mut pending_draws);
    let mut packed_lights = pack_lighting_extract_with_cookies(
        &frame.extract.lighting,
        &frame.extract.lighting.advanced_lighting.cookies,
        frame.preview().lighting_enabled,
    );
    if let Some(shadow_light_slots) = shadow_light_slots {
        shadow_light_slots
            .apply_to_packed_lights(&frame.extract.lighting, &mut packed_lights.lights);
    }
    gpu_scene.write_lights(device, &packed_lights.lights);
    let (gpu_scene_upload_report, gpu_scene_entries) = sync_gpu_scene_pending_draws(
        device,
        queue,
        encoder,
        gpu_scene,
        &mut pending_draws,
        frame.environment().baked_lighting(),
    );
    let gpu_scene_upload_report = gpu_scene_upload_report
        .with_additional_uploaded_bytes(virtual_geometry_upload_report.uploaded_bytes)
        .with_additional_uploaded_bytes(morph_upload_report.uploaded_bytes);
    let visibility_states = mesh_visibility_states(frame);
    let pending_command_cache_plan_stats =
        summarize_pending_mesh_command_cache_plan(&pending_draws, |stable_instance_key| {
            visibility_states
                .get(&stable_instance_key)
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

    let (pending_draws, prebuilt_mesh_pass_command_buffers, pending_command_cache_extraction_stats) =
        if let Some(command_cache_extraction) = command_cache_extraction {
            let extraction = extract_pending_static_mesh_command_cache_hits(
                pending_draws,
                |stable_instance_key| {
                    visibility_states
                        .get(&stable_instance_key)
                        .copied()
                        .map(PendingMeshCommandCacheVisibility::from)
                },
                |stable_instance_key| {
                    gpu_scene_entries
                        .get(&stable_instance_key)
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
                PendingMeshDrawRemainder::all(pending_draws),
                MeshPassCommandBuffers::default(),
                PendingMeshCommandCacheExtractionStats::default(),
            )
        };
    let indexed_pending_draws = pending_draws
        .into_iter()
        .map(|(original_index, pending_draw)| {
            let indirect_args_offset = indirect_args_offsets
                .get(original_index)
                .copied()
                .flatten()
                .unwrap_or((original_index as u64) * indirect_args_stride);
            let draw_indirect_args_buffer =
                indirect_args_buffers.get(original_index).cloned().flatten();
            let submission_detail = pending_draw_submission_details
                .get(original_index)
                .copied()
                .flatten()
                .or_else(|| {
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
                });
            (
                indirect_args_offset,
                draw_indirect_args_buffer,
                submission_detail,
                pending_draw,
            )
        });
    BuiltMeshDraws {
        draws: indexed_pending_draws
            .map(
                |(
                    indirect_args_offset,
                    draw_indirect_args_buffer,
                    submission_detail,
                    pending_draw,
                )| {
                    let stable_instance_key = pending_draw.stable_instance_key;
                    let synced_gpu_scene_entry = *gpu_scene_entries
                        .get(&stable_instance_key)
                        .expect("pending mesh draw must have a synchronized GPUScene entry");
                    let gpu_scene_instance_span = (
                        synced_gpu_scene_entry.entry.first_instance_index,
                        synced_gpu_scene_entry.entry.instance_count,
                    );
                    let skinned_gpu_skinning_enabled =
                        pending_draw_has_enabled_skinned_gpu_source(&pending_draw);
                    let source_selection =
                        pending_mesh_source_selection(&pending_draw, skinned_gpu_skinning_enabled);
                    let geometry_source = source_selection.geometry_source();
                    let (
                        mesh,
                        resolved_skinned_gpu_source,
                        skinned_gpu_source_uses_cpu_morphed_source,
                    ) = match source_selection {
                        PendingMeshSourceSelection::Prepared(mesh)
                        | PendingMeshSourceSelection::GpuMorphed { mesh } => (mesh, None, false),
                        PendingMeshSourceSelection::Dynamic { primitive, .. } => (
                            Arc::new(GpuMeshResource::from_asset(device, primitive)),
                            None,
                            false,
                        ),
                        PendingMeshSourceSelection::SkinnedGpu {
                            mesh,
                            source_uses_cpu_morphed_source,
                            ..
                        } => {
                            let resolved_skinned_gpu_source = Some(mesh.clone());
                            (
                                mesh,
                                resolved_skinned_gpu_source,
                                source_uses_cpu_morphed_source,
                            )
                        }
                    };
                    let previous_skinned_joint_palette = if skinned_gpu_skinning_enabled {
                        pending_draw.previous_skinned_joint_palette
                    } else {
                        None
                    };
                    let has_compatible_previous_skinned_palette =
                        skinned_gpu_skinning_enabled && previous_skinned_joint_palette.is_some();
                    let (staged_joint_palette_buffer, staged_previous_joint_palette_buffer) =
                        gpu_scene.stage_skinned_joint_palette_buffers(
                            device,
                            queue,
                            stable_instance_key,
                            pending_draw.skinned_joint_palette.as_ref(),
                            has_compatible_previous_skinned_palette,
                        );
                    let (skinned_joint_palette_buffer, previous_skinned_joint_palette_buffer) =
                        if skinned_gpu_skinning_enabled {
                            (
                                staged_joint_palette_buffer,
                                staged_previous_joint_palette_buffer,
                            )
                        } else {
                            (None, None)
                        };
                    let has_previous_velocity_transform = synced_gpu_scene_entry
                        .has_previous_velocity_transform
                        && (!skinned_gpu_skinning_enabled
                            || previous_skinned_joint_palette_buffer.is_some());
                    let material_uniform = if let Some(payload) =
                        pending_draw.material_uniform_override_payload.as_ref()
                    {
                        Arc::new(GpuMaterialUniformResource::from_payload(device, payload))
                    } else {
                        pending_draw.material_uniform
                    };
                    let mut mesh_draw = create_mesh_draw(
                        device,
                        gpu_scene,
                        material_texture_layout,
                        mesh,
                        geometry_source,
                        pending_draw.mobility,
                        pending_draw.source_entity,
                        stable_instance_key,
                        pending_draw.source_draw_ordinal,
                        pending_draw.static_state,
                        pending_draw.material_textures,
                        material_uniform,
                        pending_draw.standard_material_uniform,
                        pending_draw.pipeline_key,
                        pending_draw.common.as_ref(),
                        pending_draw.disabled_passes,
                        pending_draw.taa_reactive_mask_strength,
                        has_previous_velocity_transform,
                        pending_draw.mesh_lod,
                        pending_draw.skinned,
                        skinned_joint_palette_buffer,
                        previous_skinned_joint_palette_buffer,
                        pending_draw.previous_skinned_gpu_source,
                        resolved_skinned_gpu_source,
                        skinned_gpu_source_uses_cpu_morphed_source,
                        skinned_gpu_skinning_enabled,
                        pending_draw.first_index,
                        pending_draw.draw_index_count,
                        draw_indirect_args_buffer,
                        indirect_args_offset,
                        submission_detail,
                    )
                    .with_gpu_scene_instance_span(
                        gpu_scene_instance_span.0,
                        gpu_scene_instance_span.1,
                    )
                    .with_command_sort_input(pending_draw.command_sort_input);
                    if let Some(visibility) =
                        visibility_states.get(&stable_instance_key).copied()
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
    material_uniform_override_signature: u64,
    standard_material_uniform: usize,
    pipeline_key: crate::graphics::scene::resources::PipelineKey,
    disabled_passes: MaterialDisabledPasses,
    first_index: u32,
    draw_index_count: u32,
}

fn prepared_mesh_queue_stats_for_pending_draws(
    pending_draws: &[super::pending_mesh_draw::PendingMeshDraw],
    gpu_scene_entries: &HashMap<u64, SyncedGpuSceneEntry>,
) -> PreparedMeshQueueStats {
    summarize_prepared_mesh_queue_items(pending_draws.iter().map(|pending_draw| {
        let skinned_gpu_skinning_enabled =
            pending_draw_has_enabled_skinned_gpu_source(pending_draw);
        let has_previous_velocity_transform = gpu_scene_entries
            .get(&pending_draw.stable_instance_key)
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
            pending_draw.common.cast_shadows.casts_shadows()
                && queue_profile.phase().casts_shadow(),
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
        material_uniform_override_signature: material_uniform_override_signature(pending_draw),
        standard_material_uniform: Arc::as_ptr(&pending_draw.standard_material_uniform) as usize,
        pipeline_key: pending_draw.pipeline_key.clone(),
        disabled_passes: pending_draw.disabled_passes,
        first_index: pending_draw.first_index,
        draw_index_count: pending_draw.draw_index_count,
    }
}

fn material_uniform_override_signature(
    pending_draw: &super::pending_mesh_draw::PendingMeshDraw,
) -> u64 {
    let Some(payload) = pending_draw.material_uniform_override_payload.as_ref() else {
        return 0;
    };
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    payload.bytes.hash(&mut hasher);
    for unsupported in &payload.unsupported {
        unsupported.name.hash(&mut hasher);
        unsupported.reason.hash(&mut hasher);
    }
    let hash = hasher.finish();
    if hash == 0 {
        1
    } else {
        hash
    }
}

fn pending_mesh_identity(pending_draw: &super::pending_mesh_draw::PendingMeshDraw) -> usize {
    if let Some(mesh) = pending_draw.resolved_skinned_gpu_source.as_ref() {
        return Arc::as_ptr(mesh) as usize;
    }
    match &pending_draw.mesh {
        PendingMeshGeometry::Prepared(mesh) => Arc::as_ptr(mesh) as usize,
        PendingMeshGeometry::GpuMorphed(mesh) => Arc::as_ptr(mesh) as usize,
        PendingMeshGeometry::Dynamic(_) | PendingMeshGeometry::CpuMorphed(_) => {
            (pending_draw.stable_instance_key as usize).wrapping_mul(31)
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

fn mesh_visibility_states(frame: &ViewportRenderFrame) -> HashMap<u64, MeshVisibilityState> {
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

    debug_assert_eq!(
        frame_visibility.entities.len(),
        frame_visibility.stable_instance_keys.len(),
        "frame visibility owner and stable-key arrays must stay aligned"
    );

    frame_visibility
        .stable_instance_keys
        .iter()
        .enumerate()
        .map(|(index, stable_instance_key)| {
            let index =
                u32::try_from(index).expect("frame visibility primitive index exceeds u32 range");
            let relevance = frame_visibility
                .relevance
                .get(index as usize)
                .copied()
                .unwrap_or_default();
            (
                *stable_instance_key,
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
    use crate::core::framework::render::render_mesh_stable_instance_key;
    use crate::core::framework::scene::Mobility;
    use crate::core::math::{UVec2, Vec4};
    use crate::graphics::visibility::{
        FrameVisibility, ViewCullingStats, ViewVisibilityContext, VisibilityBounds,
        VisibilityViewKey,
    };
    use crate::graphics::ViewportRenderFrame;

    #[test]
    fn mesh_visibility_states_keep_sibling_primitives_independent() {
        let main_visible_stable_instance_key = render_mesh_stable_instance_key(1, 0);
        let shadow_visible_stable_instance_key = render_mesh_stable_instance_key(1, 1);
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
                    environment: crate::core::framework::render::EnvironmentExtract::default(),
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
            entities: vec![1, 1],
            stable_instance_keys: vec![
                main_visible_stable_instance_key,
                shadow_visible_stable_instance_key,
            ],
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
                RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
                RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
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

        assert_eq!(states.len(), 2);
        let main_receiver = states
            .get(&main_visible_stable_instance_key)
            .expect("main-view receiver state");
        assert!(main_receiver.main_view_visible);
        assert!(!main_receiver.shadow_view_visible);

        let shadow_only_caster = states
            .get(&shadow_visible_stable_instance_key)
            .expect("shadow-only caster state");
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
