use std::collections::HashSet;

use crate::core::framework::render::{
    RenderGpuSceneUploadPath, RenderVirtualGeometryExecutionDraw,
    RenderVirtualGeometryExecutionSegment, RenderVirtualGeometryExecutionState,
};

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
    pub(crate) gpu_morphed_source_draw_count: usize,
    pub(crate) gpu_skinned_morphed_source_draw_count: usize,
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

impl From<&RenderVirtualGeometryExecutionSegment> for VirtualGeometryExecutionSegmentKey {
    fn from(segment: &RenderVirtualGeometryExecutionSegment) -> Self {
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
