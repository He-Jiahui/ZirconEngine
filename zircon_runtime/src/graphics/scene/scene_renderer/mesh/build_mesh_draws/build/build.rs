use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::core::framework::render::{CastShadowsMode, PrimitiveRelevance};
use crate::core::framework::scene::EntityId;
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::gpu_scene::{GpuScene, GpuScenePreparedUpload, GpuSceneUploadReport};
use crate::graphics::scene::resources::{
    GpuMaterialUniformResource, GpuMeshResource, MaterialDisabledPasses, ResourceStreamer,
};
use crate::graphics::scene::scene_renderer::lighting::light_buffer::pack_lighting_extract_with_cookies;
use crate::graphics::scene::scene_renderer::shadow::ShadowLightSlotAssignments;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

use super::super::super::mesh_draw::VirtualGeometrySubmissionDetail;
use super::super::super::mesh_draw::{
    MaterialTextureSet, MeshDraw, MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
};
use super::super::super::mesh_pass::MeshPassCommandBuffers;
use super::super::super::prepared_queue::{
    PreparedMeshQueueStats, summarize_prepared_mesh_queue_items,
};
use super::super::MeshHitProxyTokenSource;
use super::super::create_mesh_draw::{create_mesh_draw, record_material_binding_build_profile};
use super::super::indexed_indirect_args::IndexedIndirectArgs;
use super::build_mesh_draw_build_context::build_mesh_draw_build_context;
use super::collect_pending_draws::{
    collect_pending_draws, collect_pending_draws_with_published_pipeline_requirements,
};
use super::geometry_source_selection::{
    PendingMeshSourceSelection, pending_draw_has_enabled_skinned_gpu_source,
    pending_mesh_draw_geometry_source, pending_mesh_source_selection,
};
use super::gpu_scene_sync::{SyncedGpuSceneEntry, sync_gpu_scene_pending_draws};
use super::material_context_admission::select_material_generations_for_context;
use super::material_draw_selection::MaterialDrawSelection;
use super::material_pipeline_requirements::{
    MaterialPipelineFeatureSet, MaterialPipelineRequirementCensus,
    collect_material_pipeline_requirements,
};
use super::morph_payload_upload::upload_morph_payloads;
use super::pending_command_cache_extract::{
    PendingMeshCommandCacheExtractionContext, PendingMeshCommandCacheExtractionStats,
    PendingMeshDrawRemainder, extract_pending_static_mesh_command_cache_hits,
};
use super::pending_command_cache_plan::{
    PendingMeshCommandCachePlanStats, PendingMeshCommandCacheVisibility,
    summarize_pending_mesh_command_cache_plan,
};
use super::pending_mesh_draw::{PendingMeshGeometry, PendingSkinnedGpuSource};
use super::virtual_geometry_indirect::build_virtual_geometry_indirect_draw_plan;
use super::virtual_geometry_resident_upload::upload_virtual_geometry_resident_payloads;

pub(crate) struct BuiltMeshDraws {
    draws: Vec<MeshDraw>,
    prepared_mesh_queue_stats: PreparedMeshQueueStats,
    prebuilt_mesh_pass_command_buffers: MeshPassCommandBuffers,
    gpu_scene_prepared_upload: Option<GpuScenePreparedUpload>,
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
    material_pipeline_requirements: MaterialPipelineRequirementCensus,
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

    pub(crate) fn take_gpu_scene_prepared_upload(&mut self) -> GpuScenePreparedUpload {
        self.gpu_scene_prepared_upload
            .take()
            .expect("built mesh draws must retain their prepared GPU Scene upload")
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

    pub(crate) fn take_material_pipeline_requirements(
        &mut self,
    ) -> MaterialPipelineRequirementCensus {
        std::mem::take(&mut self.material_pipeline_requirements)
    }
}

pub(crate) fn build_mesh_draws(
    backend: &RenderBackend,
    encoder: &mut wgpu::CommandEncoder,
    material_texture_layout: &wgpu::BindGroupLayout,
    gpu_scene: &mut GpuScene,
    streamer: &mut ResourceStreamer,
    mesh_pipelines: &mut super::super::super::MeshPipelineCache,
    frame: &ViewportRenderFrame,
    virtual_geometry_enabled: bool,
    volumetric_fog_enabled: bool,
    material_pipeline_features: MaterialPipelineFeatureSet,
    direct_lighting_preparation: Option<bool>,
    shadow_light_slots: Option<&ShadowLightSlotAssignments>,
    command_cache_extraction: Option<PendingMeshCommandCacheExtractionContext<'_>>,
    hit_proxy_tokens: Option<&dyn MeshHitProxyTokenSource>,
) -> Result<BuiltMeshDraws, GraphicsError> {
    let device = &backend.device;
    let build_context = build_mesh_draw_build_context(
        frame,
        virtual_geometry_enabled,
        material_pipeline_features.reverses_view_raster_winding(),
    );
    let published_selection = MaterialDrawSelection::default();
    let (mut pending_draws, current_material_pipeline_requirements) =
        collect_pending_draws_with_published_pipeline_requirements(
            streamer,
            frame,
            &build_context,
            gpu_scene,
            &published_selection,
            material_pipeline_features,
            frame.shader_quality(),
            volumetric_fog_enabled,
        );
    let (material_selection, _) = select_material_generations_for_context(
        device,
        streamer,
        mesh_pipelines,
        &pending_draws,
        current_material_pipeline_requirements,
        material_pipeline_features,
        frame.shader_quality(),
        volumetric_fog_enabled,
    )?;
    if material_selection.has_overrides() {
        pending_draws = collect_pending_draws(
            streamer,
            frame,
            &build_context,
            gpu_scene,
            &material_selection,
        );
    }
    for pending_draw in &mut pending_draws {
        pending_draw.material.pipeline_key.volumetric_fog = volumetric_fog_enabled;
        pending_draw
            .material
            .textures
            .set_max_anisotropy(frame.texture_max_anisotropy());
        refresh_pending_mesh_material_submission_revision(pending_draw);
    }
    if let Some(hit_proxy_tokens) = hit_proxy_tokens {
        pending_draws.retain(|pending_draw| {
            hit_proxy_tokens
                .token_for_instance(pending_draw.stable_instance_key)
                .is_some_and(|token| token != 0)
        });
    }
    let indirect_plan = build_virtual_geometry_indirect_draw_plan(
        device,
        frame,
        virtual_geometry_enabled,
        &mut pending_draws,
    );
    let virtual_geometry_upload = upload_virtual_geometry_resident_payloads(
        device,
        gpu_scene,
        virtual_geometry_enabled,
        frame.virtual_geometry_debug_snapshot.as_ref(),
    );
    let virtual_geometry_scene_counts = virtual_geometry_upload.scene_data_counts();
    let morph_upload = upload_morph_payloads(device, gpu_scene, &mut pending_draws);
    if let Some(direct_lighting_enabled) = direct_lighting_preparation {
        let mut packed_lights = pack_lighting_extract_with_cookies(
            &frame.extract.lighting,
            &frame.extract.lighting.advanced_lighting.cookies,
            direct_lighting_enabled,
        );
        if let Some(shadow_light_slots) = shadow_light_slots {
            shadow_light_slots
                .apply_to_packed_lights(&frame.extract.lighting, &mut packed_lights.lights);
        }
        gpu_scene.write_lights(device, &packed_lights.lights);
    }
    let (mut gpu_scene_prepared_upload, gpu_scene_entries) = sync_gpu_scene_pending_draws(
        backend,
        encoder,
        gpu_scene,
        &mut pending_draws,
        virtual_geometry_scene_counts,
        frame.environment().baked_lighting(),
        hit_proxy_tokens,
    )?;
    gpu_scene_prepared_upload.append_virtual_geometry_upload(virtual_geometry_upload);
    gpu_scene_prepared_upload.append_morph_upload(morph_upload);
    let gpu_scene_upload_report = gpu_scene_prepared_upload.report();
    let visibility_states = mesh_visibility_states(frame);
    let material_pipeline_requirements = collect_material_pipeline_requirements(
        &pending_draws,
        streamer,
        material_pipeline_features,
        frame.shader_quality(),
        volumetric_fog_enabled,
    );
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
                mesh_pipelines,
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
    let (residual_draw_count, residual_draw_count_upper_bound) = indexed_pending_draws.size_hint();
    debug_assert_eq!(
        residual_draw_count_upper_bound,
        Some(residual_draw_count),
        "pending draw remainder must preserve an exact size hint"
    );
    let mut override_uniform_buffer_creation_count = 0;
    let draws: Vec<MeshDraw> = {
        crate::profile_scope!("render", "material", "binding.build_residual_draws");
        indexed_pending_draws
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
                    let has_skinned_joint_palette_upload = skinned_gpu_skinning_enabled
                        && pending_draw.skinned_joint_palette.is_some();
                    let has_previous_skinned_joint_palette_upload = skinned_gpu_skinning_enabled
                        && pending_draw.previous_skinned_joint_palette.is_some();
                    let has_previous_velocity_transform = synced_gpu_scene_entry
                        .has_previous_velocity_transform
                        && (!skinned_gpu_skinning_enabled
                            || has_previous_skinned_joint_palette_upload);
                    let material_uniform = if let Some(payload) =
                        pending_draw.material.uniform_override_payload.as_ref()
                    {
                        override_uniform_buffer_creation_count += 1;
                        Arc::new(GpuMaterialUniformResource::from_payload(device, payload))
                    } else {
                        pending_draw.material.uniform
                    };
                    let mut mesh_draw = create_mesh_draw(
                        device,
                        material_texture_layout,
                        mesh,
                        geometry_source,
                        pending_draw.mobility,
                        pending_draw.source_entity,
                        pending_draw.material.resource_id,
                        stable_instance_key,
                        pending_draw.source_draw_ordinal,
                        pending_draw.static_state,
                        pending_draw.material.textures,
                        material_uniform,
                        pending_draw.material.standard_uniform,
                        pending_draw.material.pipeline_key,
                        pending_draw.material.common.as_ref(),
                        pending_draw.material.disabled_passes,
                        pending_draw.material.taa_reactive_mask_strength,
                        pending_draw.material.half_resolution_transparency,
                        has_previous_velocity_transform,
                        pending_draw.mesh_lod,
                        pending_draw.skinned,
                        has_skinned_joint_palette_upload,
                        has_previous_skinned_joint_palette_upload,
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
                    if let Some(visibility) = visibility_states.get(&stable_instance_key).copied() {
                        mesh_draw = mesh_draw.with_visibility(
                            visibility.relevance,
                            visibility.main_view_visible,
                            visibility.shadow_view_visible,
                        );
                    }
                    mesh_draw
                },
            )
            .collect()
    };
    record_material_binding_build_profile(
        residual_draw_count,
        override_uniform_buffer_creation_count,
    );
    Ok(BuiltMeshDraws {
        draws,
        prepared_mesh_queue_stats,
        prebuilt_mesh_pass_command_buffers,
        gpu_scene_prepared_upload: Some(gpu_scene_prepared_upload),
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
        material_pipeline_requirements,
    })
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
    clearcoat_normal_texture: usize,
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
                pending_draw.material.pipeline_key.is_transparent(),
                pending_draw.material.pipeline_key.is_alpha_mask(),
            ),
            geometry_source,
            pending_draw.mobility,
            false,
            skinned_gpu_skinning_enabled,
            pending_draw.mesh_lod.is_some(),
        );
        (
            queue_profile,
            pending_draw.material.common.cast_shadows.casts_shadows()
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
            &pending_draw.material.textures,
            |textures| &textures.base_color,
        ),
        normal_texture: material_texture_identity(&pending_draw.material.textures, |textures| {
            &textures.normal
        }),
        metallic_roughness_texture: material_texture_identity(
            &pending_draw.material.textures,
            |textures| &textures.metallic_roughness,
        ),
        occlusion_texture: material_texture_identity(&pending_draw.material.textures, |textures| {
            &textures.occlusion
        }),
        emissive_texture: material_texture_identity(&pending_draw.material.textures, |textures| {
            &textures.emissive
        }),
        clearcoat_normal_texture: material_texture_identity(
            &pending_draw.material.textures,
            |textures| &textures.clearcoat_normal,
        ),
        material_uniform: Arc::as_ptr(&pending_draw.material.uniform) as usize,
        material_uniform_override_signature: material_uniform_override_signature(pending_draw),
        standard_material_uniform: Arc::as_ptr(&pending_draw.material.standard_uniform) as usize,
        pipeline_key: pending_draw.material.pipeline_key.clone(),
        disabled_passes: pending_draw.material.disabled_passes,
        first_index: pending_draw.first_index,
        draw_index_count: pending_draw.draw_index_count,
    }
}

fn refresh_pending_mesh_material_submission_revision(
    pending_draw: &mut super::pending_mesh_draw::PendingMeshDraw,
) {
    let source_revision = pending_draw.static_state.material_revision;
    pending_draw.static_state.material_revision = material_submission_revision(
        source_revision,
        &pending_draw.material.pipeline_key,
        [
            pending_draw.material.textures.base_color.identity(),
            pending_draw.material.textures.normal.identity(),
            pending_draw.material.textures.metallic_roughness.identity(),
            pending_draw.material.textures.occlusion.identity(),
            pending_draw.material.textures.emissive.identity(),
            pending_draw.material.textures.clearcoat_normal.identity(),
        ],
        Arc::as_ptr(&pending_draw.material.uniform) as usize,
        Arc::as_ptr(&pending_draw.material.standard_uniform) as usize,
        pending_draw.material.common.cast_shadows,
    );
}

fn material_submission_revision(
    source_revision: u64,
    pipeline_key: &crate::graphics::scene::resources::PipelineKey,
    texture_identities: [usize; 6],
    material_uniform_identity: usize,
    standard_material_uniform_identity: usize,
    cast_shadows: CastShadowsMode,
) -> u64 {
    if source_revision == 0 {
        return 0;
    }
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source_revision.hash(&mut hasher);
    pipeline_key.hash(&mut hasher);
    texture_identities.hash(&mut hasher);
    material_uniform_identity.hash(&mut hasher);
    standard_material_uniform_identity.hash(&mut hasher);
    cast_shadows.hash(&mut hasher);
    let hash = hasher.finish();
    if hash == 0 { 1 } else { hash }
}

fn material_uniform_override_signature(
    pending_draw: &super::pending_mesh_draw::PendingMeshDraw,
) -> u64 {
    let Some(payload) = pending_draw.material.uniform_override_payload.as_ref() else {
        return 0;
    };
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    payload.bytes.hash(&mut hasher);
    for unsupported in &payload.unsupported {
        unsupported.name.hash(&mut hasher);
        unsupported.reason.hash(&mut hasher);
    }
    let hash = hasher.finish();
    if hash == 0 { 1 } else { hash }
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
    use crate::core::framework::render::render_mesh_stable_instance_key;
    use crate::core::framework::render::{
        CorePipelineKind, FallbackSkyboxKind, PreviewEnvironmentExtract, PrimitiveRelevance,
        RenderFrameExtract, RenderLayerSet, RenderMaterialAlphaMode, RenderOverlayExtract,
        RenderSceneGeometryExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
        ViewportCameraSnapshot,
    };
    use crate::core::framework::scene::Mobility;
    use crate::core::math::{UVec2, Vec4};
    use crate::graphics::ViewportRenderFrame;
    use crate::graphics::scene::resources::default_pipeline_key;
    use crate::graphics::visibility::{
        FrameVisibility, ViewCullingStats, ViewVisibilityContext, VisibilityBounds,
        VisibilityViewKey,
    };

    fn production_source() -> &'static str {
        include_str!("build.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("mesh draw builder should retain a test-module boundary")
    }

    #[test]
    fn omitted_direct_light_preparation_skips_packing_and_gpu_light_buffer_writes() {
        let source = production_source();
        let build_function = source
            .split("pub(crate) fn build_mesh_draws(")
            .nth(1)
            .expect("mesh draw builder function");
        let omitted_branch = build_function
            .split("if let Some(direct_lighting_enabled) = direct_lighting_preparation {")
            .nth(1)
            .and_then(|source| source.split("let (gpu_scene_upload_report").next())
            .expect("direct-light preparation branch");

        assert!(omitted_branch.contains("pack_lighting_extract_with_cookies("));
        assert!(omitted_branch.contains("gpu_scene.write_lights("));
        assert!(
            !build_function[..build_function
                .find("if let Some(direct_lighting_enabled) = direct_lighting_preparation {")
                .expect("direct-light preparation gate")]
                .contains("pack_lighting_extract_with_cookies("),
            "light packing must stay behind the profile-controlled preparation gate"
        );
        assert!(
            !build_function[..build_function
                .find("if let Some(direct_lighting_enabled) = direct_lighting_preparation {")
                .expect("direct-light preparation gate")]
                .contains("gpu_scene.write_lights("),
            "GPU light-buffer writes must stay behind the profile-controlled preparation gate"
        );
    }

    #[test]
    fn material_submission_revision_tracks_final_pipeline_and_binding_identities() {
        let pipeline = default_pipeline_key();
        let textures = [11, 13, 17, 19, 23, 29];
        let revision = super::material_submission_revision(
            7,
            &pipeline,
            textures,
            31,
            37,
            crate::core::framework::render::CastShadowsMode::On,
        );

        let mut changed_pipeline = pipeline.clone();
        changed_pipeline.shader_dependency_revision = 41;
        assert_ne!(
            revision,
            super::material_submission_revision(
                7,
                &changed_pipeline,
                textures,
                31,
                37,
                crate::core::framework::render::CastShadowsMode::On,
            ),
            "transitive shader generations must invalidate cached submission payloads"
        );

        let mut changed_textures = textures;
        changed_textures[0] = 43;
        assert_ne!(
            revision,
            super::material_submission_revision(
                7,
                &pipeline,
                changed_textures,
                31,
                37,
                crate::core::framework::render::CastShadowsMode::On,
            ),
            "mip residency resource replacement must invalidate cached material bind groups"
        );
        assert_ne!(
            revision,
            super::material_submission_revision(
                7,
                &pipeline,
                textures,
                31,
                37,
                crate::core::framework::render::CastShadowsMode::TwoSided,
            ),
            "effective renderer shadow raster mode must invalidate cached commands"
        );
        assert_eq!(
            super::material_submission_revision(
                0,
                &pipeline,
                textures,
                31,
                37,
                crate::core::framework::render::CastShadowsMode::On,
            ),
            0,
            "missing source authority must not become cacheable through process-local identities"
        );
    }

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
