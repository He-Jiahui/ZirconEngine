use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use super::mesh_draw::{
    MeshDraw, MeshDrawBatchKey, MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
};
use crate::core::framework::render::{
    RenderGpuSceneUploadPath, RenderVirtualGeometryExecutionDraw,
    RenderVirtualGeometryExecutionState,
};

mod stats_bridge;

#[cfg(test)]
mod stats_bridge_tests;

pub(crate) struct PreparedMeshQueue {
    stats: PreparedMeshQueueStats,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PreparedMeshQueueStats {
    pub(crate) draw_count: usize,
    pub(crate) opaque_draw_count: usize,
    pub(crate) alpha_mask_draw_count: usize,
    pub(crate) transparent_draw_count: usize,
    pub(crate) early_z_draw_count: usize,
    pub(crate) shadow_caster_draw_count: usize,
    pub(crate) alpha_mask_shadow_caster_draw_count: usize,
    pub(crate) prepared_geometry_draw_count: usize,
    pub(crate) dynamic_geometry_draw_count: usize,
    pub(crate) skinned_draw_count: usize,
    pub(crate) skinned_palette_upload_count: usize,
    pub(crate) skinned_previous_palette_upload_count: usize,
    pub(crate) skinned_gpu_source_candidate_count: usize,
    pub(crate) skinned_gpu_cpu_morphed_source_candidate_count: usize,
    pub(crate) skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count: usize,
    pub(crate) skinned_gpu_skinning_draw_count: usize,
    pub(crate) skinned_gpu_velocity_draw_count: usize,
    pub(crate) indirect_draw_count: usize,
    pub(crate) lod_draw_count: usize,
    pub(crate) static_batch_candidate_group_count: usize,
    pub(crate) static_batch_candidate_draw_count: usize,
    pub(crate) dynamic_batch_candidate_group_count: usize,
    pub(crate) dynamic_batch_candidate_draw_count: usize,
    pub(crate) gpu_instancing_candidate_group_count: usize,
    pub(crate) gpu_instancing_candidate_draw_count: usize,
    pub(crate) indirect_batch_count: usize,
    pub(crate) indirect_batched_draw_count: usize,
    pub(crate) indirect_fallback_draw_count: usize,
    pub(crate) indirect_args_count: usize,
    pub(crate) virtual_geometry_indirect_draw_count: usize,
    pub(crate) virtual_geometry_indirect_buffer_count: usize,
    pub(crate) virtual_geometry_indirect_args_count: usize,
    pub(crate) virtual_geometry_indirect_segment_count: usize,
    pub(crate) virtual_geometry_execution_draw_count: usize,
    pub(crate) virtual_geometry_execution_segment_count: usize,
    pub(crate) virtual_geometry_execution_page_count: usize,
    pub(crate) virtual_geometry_execution_resident_segment_count: usize,
    pub(crate) virtual_geometry_execution_pending_segment_count: usize,
    pub(crate) virtual_geometry_execution_missing_segment_count: usize,
    pub(crate) virtual_geometry_execution_repeated_draw_count: usize,
    pub(crate) gpu_scene_primitive_count: u32,
    pub(crate) gpu_scene_instance_count: u32,
    pub(crate) gpu_scene_dirty_entry_count: usize,
    pub(crate) gpu_scene_uploaded_bytes: u64,
    pub(crate) gpu_scene_upload_path: RenderGpuSceneUploadPath,
    pub(crate) gpu_scene_free_span_count: usize,
    pub(crate) gpu_scene_primitive_upload_range_count: usize,
    pub(crate) gpu_scene_instance_upload_range_count: usize,
    pub(crate) previous_velocity_transform_draw_count: usize,
    pub(crate) missing_velocity_transform_draw_count: usize,
    pub(crate) command_count: usize,
    pub(crate) depth_prepass_command_count: usize,
    pub(crate) shadow_command_count: usize,
    pub(crate) opaque_command_count: usize,
    pub(crate) alpha_mask_command_count: usize,
    pub(crate) transparent_command_count: usize,
    pub(crate) velocity_command_count: usize,
    pub(crate) taa_reactive_mask_command_count: usize,
    pub(crate) cached_command_hit_count: usize,
    pub(crate) command_rebuild_count: usize,
    pub(crate) dynamic_command_count: usize,
    pub(crate) pending_static_command_cache_draw_candidate_count: usize,
    pub(crate) pending_static_command_cache_phase_candidate_count: usize,
    pub(crate) pending_static_command_cache_depth_prepass_candidate_count: usize,
    pub(crate) pending_static_command_cache_shadow_candidate_count: usize,
    pub(crate) pending_static_command_cache_opaque_candidate_count: usize,
    pub(crate) pending_static_command_cache_alpha_mask_candidate_count: usize,
    pub(crate) pre_mesh_draw_static_command_cache_skipped_draw_count: usize,
    pub(crate) pre_mesh_draw_static_command_cache_skipped_phase_count: usize,
    pub(crate) pre_mesh_draw_static_command_cache_visibility_pruned_draw_count: usize,
    pub(crate) pre_mesh_draw_static_command_cache_residual_material_phase_draw_count: usize,
    pub(crate) pre_mesh_draw_static_command_cache_residual_rebuild_input_missing_draw_count: usize,
    pub(crate) pre_mesh_draw_static_command_cache_residual_rebuild_rejected_draw_count: usize,
    pub(crate) cache_miss_count: usize,
    pub(crate) cache_invalidated_transform_count: usize,
    pub(crate) cache_invalidated_geometry_count: usize,
    pub(crate) cache_invalidated_material_count: usize,
    pub(crate) state_change_count: usize,
    pub(crate) bind_skip_count: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PreparedMeshVirtualGeometryIndirectStats {
    pub(crate) draw_count: usize,
    pub(crate) buffer_count: usize,
    pub(crate) args_count: usize,
    pub(crate) segment_count: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PreparedMeshVirtualGeometryExecutionStats {
    pub(crate) draw_count: usize,
    pub(crate) segment_count: usize,
    pub(crate) page_count: usize,
    pub(crate) resident_segment_count: usize,
    pub(crate) pending_segment_count: usize,
    pub(crate) missing_segment_count: usize,
    pub(crate) repeated_draw_count: usize,
}

impl PreparedMeshVirtualGeometryExecutionStats {
    pub(crate) fn from_execution_draws(
        execution_draws: impl IntoIterator<Item = RenderVirtualGeometryExecutionDraw>,
    ) -> Self {
        let mut stats = Self::default();
        let mut segments = HashSet::new();
        let mut pages = HashSet::new();

        for draw in execution_draws {
            if !draw.uses_indirect_draw || draw.execution_selection_key.is_none() {
                continue;
            }

            stats.draw_count += 1;
            let segment = draw.execution_segment;
            let key = VirtualGeometryExecutionSegmentKey::from(&segment);
            if !segments.insert(key) {
                continue;
            }

            stats.segment_count += 1;
            pages.insert(segment.page_id);
            match segment.state {
                RenderVirtualGeometryExecutionState::Resident => stats.resident_segment_count += 1,
                RenderVirtualGeometryExecutionState::PendingUpload => {
                    stats.pending_segment_count += 1
                }
                RenderVirtualGeometryExecutionState::Missing => stats.missing_segment_count += 1,
            }
        }

        stats.page_count = pages.len();
        stats.repeated_draw_count = stats.draw_count.saturating_sub(stats.segment_count);
        stats
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct VirtualGeometryExecutionSegmentKey {
    instance_index: Option<u32>,
    entity: u64,
    page_id: u32,
    cluster_start_ordinal: u32,
    cluster_span_count: u32,
    cluster_total_count: u32,
    submission_slot: Option<u32>,
    state: u32,
    lineage_depth: u32,
    lod_level: u8,
    frontier_rank: u32,
}

impl From<&crate::core::framework::render::RenderVirtualGeometryExecutionSegment>
    for VirtualGeometryExecutionSegmentKey
{
    fn from(
        segment: &crate::core::framework::render::RenderVirtualGeometryExecutionSegment,
    ) -> Self {
        Self {
            instance_index: segment.instance_index,
            entity: segment.entity,
            page_id: segment.page_id,
            cluster_start_ordinal: segment.cluster_start_ordinal,
            cluster_span_count: segment.cluster_span_count,
            cluster_total_count: segment.cluster_total_count,
            submission_slot: segment.submission_slot,
            state: encode_virtual_geometry_execution_state(segment.state),
            lineage_depth: segment.lineage_depth,
            lod_level: segment.lod_level,
            frontier_rank: segment.frontier_rank,
        }
    }
}

fn encode_virtual_geometry_execution_state(state: RenderVirtualGeometryExecutionState) -> u32 {
    match state {
        RenderVirtualGeometryExecutionState::Resident => 0,
        RenderVirtualGeometryExecutionState::PendingUpload => 1,
        RenderVirtualGeometryExecutionState::Missing => 2,
    }
}

pub(crate) fn prepare_mesh_queue(draws: &[MeshDraw]) -> PreparedMeshQueue {
    let stats = summarize_prepared_mesh_queue_items::<MeshDrawBatchKey>(draws.iter().map(|draw| {
        (
            draw.queue_profile(),
            draw.casts_shadow(),
            draw.has_previous_velocity_transform(),
            draw.is_skinned(),
            draw.has_skinned_joint_palette_upload(),
            draw.has_previous_skinned_joint_palette_upload(),
            draw.has_skinned_gpu_source_candidate(),
            draw.has_skinned_gpu_cpu_morphed_source_candidate(),
            draw.uses_skinned_gpu_skinning(),
            draw.batch_key(),
        )
    }));

    PreparedMeshQueue { stats }
}

impl PreparedMeshQueue {
    pub(crate) fn stats(&self) -> PreparedMeshQueueStats {
        self.stats
    }
}

pub(crate) fn summarize_prepared_mesh_queue_items<K>(
    items: impl IntoIterator<
        Item = (
            MeshDrawQueueProfile,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            K,
        ),
    >,
) -> PreparedMeshQueueStats
where
    K: Clone + Eq + Hash,
{
    let mut stats = PreparedMeshQueueStats::default();
    let mut static_batch_groups = HashMap::<K, usize>::new();
    let mut dynamic_batch_groups = HashMap::<K, usize>::new();
    let mut gpu_instancing_groups = HashMap::<K, usize>::new();

    for (
        profile,
        casts_shadow,
        has_previous_velocity_transform,
        is_skinned,
        has_skinned_joint_palette_upload,
        has_previous_skinned_joint_palette_upload,
        has_skinned_gpu_source_candidate,
        has_skinned_gpu_cpu_morphed_source_candidate,
        uses_skinned_gpu_skinning,
        key,
    ) in items
    {
        stats.draw_count += 1;
        let phase = profile.phase();
        match phase {
            MeshDrawQueuePhase::Opaque => stats.opaque_draw_count += 1,
            MeshDrawQueuePhase::AlphaMask => stats.alpha_mask_draw_count += 1,
            MeshDrawQueuePhase::Transparent => stats.transparent_draw_count += 1,
        }
        if casts_shadow {
            stats.shadow_caster_draw_count += 1;
            if matches!(phase, MeshDrawQueuePhase::AlphaMask) {
                stats.alpha_mask_shadow_caster_draw_count += 1;
            }
        }
        if profile.early_z_eligible() {
            stats.early_z_draw_count += 1;
        }
        match profile.geometry_source() {
            MeshDrawGeometrySource::Prepared => stats.prepared_geometry_draw_count += 1,
            MeshDrawGeometrySource::Dynamic | MeshDrawGeometrySource::DynamicGpuSkinningSource => {
                stats.dynamic_geometry_draw_count += 1
            }
        }
        if profile.uses_indirect_draw() {
            stats.indirect_draw_count += 1;
        }
        if profile.uses_mesh_lod() {
            stats.lod_draw_count += 1;
        }
        if is_skinned {
            stats.skinned_draw_count += 1;
        }
        if has_skinned_joint_palette_upload {
            stats.skinned_palette_upload_count += 1;
        }
        if has_previous_skinned_joint_palette_upload {
            stats.skinned_previous_palette_upload_count += 1;
        }
        if has_skinned_gpu_source_candidate {
            stats.skinned_gpu_source_candidate_count += 1;
        }
        if has_skinned_gpu_cpu_morphed_source_candidate {
            stats.skinned_gpu_cpu_morphed_source_candidate_count += 1;
        }
        let missing_cpu_morphed_previous_shape_velocity = profile.velocity_history_eligible()
            && uses_skinned_gpu_skinning
            && has_skinned_gpu_cpu_morphed_source_candidate
            && !has_previous_velocity_transform;
        if uses_skinned_gpu_skinning {
            stats.skinned_gpu_skinning_draw_count += 1;
            if has_previous_velocity_transform {
                stats.skinned_gpu_velocity_draw_count += 1;
            }
        }
        if profile.velocity_history_eligible() {
            if has_previous_velocity_transform {
                stats.previous_velocity_transform_draw_count += 1;
            } else if missing_cpu_morphed_previous_shape_velocity {
                stats.skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count += 1;
            } else {
                stats.missing_velocity_transform_draw_count += 1;
            }
        }
        if profile.static_batch_eligible() {
            *static_batch_groups.entry(key.clone()).or_default() += 1;
        }
        if profile.dynamic_batch_eligible() {
            *dynamic_batch_groups.entry(key.clone()).or_default() += 1;
        }
        if profile.gpu_instancing_eligible() {
            *gpu_instancing_groups.entry(key).or_default() += 1;
        }
    }

    let (groups, draws) = repeated_group_stats(static_batch_groups.values().copied());
    stats.static_batch_candidate_group_count = groups;
    stats.static_batch_candidate_draw_count = draws;
    let (groups, draws) = repeated_group_stats(dynamic_batch_groups.values().copied());
    stats.dynamic_batch_candidate_group_count = groups;
    stats.dynamic_batch_candidate_draw_count = draws;
    let (groups, draws) = repeated_group_stats(gpu_instancing_groups.values().copied());
    stats.gpu_instancing_candidate_group_count = groups;
    stats.gpu_instancing_candidate_draw_count = draws;

    stats
}

fn repeated_group_stats(group_sizes: impl IntoIterator<Item = usize>) -> (usize, usize) {
    group_sizes
        .into_iter()
        .filter(|size| *size > 1)
        .fold((0, 0), |(groups, draws), size| (groups + 1, draws + size))
}

#[cfg(test)]
mod tests;
