use super::super::build_mesh_draws::{
    PendingMeshCommandCacheExtractionStats, PendingMeshCommandCachePlanStats,
};
use super::super::mesh_pass::{MeshDrawReplayStats, MeshPassCommandBufferStats};
use super::{
    PreparedMeshQueueStats, PreparedMeshVirtualGeometryExecutionStats,
    PreparedMeshVirtualGeometryIndirectStats,
};

mod gpu_scene_stats;

impl PreparedMeshQueueStats {
    pub(crate) fn with_pending_command_cache_extraction_stats(
        mut self,
        stats: PendingMeshCommandCacheExtractionStats,
    ) -> Self {
        self.pre_mesh_draw_static_command_cache_skipped_draw_count = stats.skipped_mesh_draw_count;
        self.pre_mesh_draw_static_command_cache_skipped_phase_count = stats.skipped_phase_count;
        self.pre_mesh_draw_static_command_cache_visibility_pruned_draw_count =
            stats.visibility_pruned_mesh_draw_count;
        self.pre_mesh_draw_static_command_cache_residual_material_phase_draw_count =
            stats.residual_material_phase_draw_count;
        self.pre_mesh_draw_static_command_cache_residual_rebuild_input_missing_draw_count =
            stats.residual_rebuild_input_missing_draw_count;
        self.pre_mesh_draw_static_command_cache_residual_rebuild_rejected_draw_count =
            stats.residual_rebuild_rejected_draw_count;
        self
    }

    pub(crate) fn with_pending_command_cache_plan_stats(
        mut self,
        stats: PendingMeshCommandCachePlanStats,
    ) -> Self {
        self.pending_static_command_cache_draw_candidate_count =
            stats.static_command_cache_draw_candidate_count;
        self.pending_static_command_cache_phase_candidate_count =
            stats.static_command_cache_phase_candidate_count;
        self.pending_static_command_cache_depth_prepass_candidate_count =
            stats.static_command_cache_depth_prepass_candidate_count;
        self.pending_static_command_cache_shadow_candidate_count =
            stats.static_command_cache_shadow_candidate_count;
        self.pending_static_command_cache_opaque_candidate_count =
            stats.static_command_cache_opaque_candidate_count;
        self.pending_static_command_cache_alpha_mask_candidate_count =
            stats.static_command_cache_alpha_mask_candidate_count;
        self
    }

    pub(crate) fn with_mesh_pass_command_buffer_stats(
        mut self,
        command_stats: MeshPassCommandBufferStats,
    ) -> Self {
        self.command_count = command_stats.command_count;
        self.depth_prepass_command_count = command_stats.depth_prepass_command_count;
        self.shadow_command_count = command_stats.shadow_command_count;
        self.opaque_command_count = command_stats.opaque_command_count;
        self.alpha_mask_command_count = command_stats.alpha_mask_command_count;
        self.advanced_pbr_opaque_command_count = command_stats.advanced_pbr_opaque_command_count;
        self.transmission_command_count = command_stats.transmission_command_count;
        self.transparent_command_count = command_stats.transparent_command_count;
        self.velocity_command_count = command_stats.velocity_command_count;
        self.taa_reactive_mask_command_count = command_stats.taa_reactive_mask_command_count;
        self.cached_command_hit_count = command_stats.cached_command_hit_count;
        self.command_rebuild_count = command_stats.command_rebuild_count;
        self.dynamic_command_count = command_stats.dynamic_command_count;
        self.cache_miss_count = command_stats.cache_miss_count;
        self.cache_invalidated_transform_count = command_stats.cache_invalidated_transform_count;
        self.cache_invalidated_geometry_count = command_stats.cache_invalidated_geometry_count;
        self.cache_invalidated_material_count = command_stats.cache_invalidated_material_count;
        self.indirect_batch_count = command_stats.indirect_batch_count;
        self.indirect_batched_draw_count = command_stats.indirect_batched_draw_count;
        self.indirect_fallback_draw_count = command_stats.indirect_fallback_draw_count;
        self.indirect_args_count = command_stats.indirect_args_count;
        self
    }

    pub(crate) fn with_virtual_geometry_indirect_stats(
        mut self,
        stats: PreparedMeshVirtualGeometryIndirectStats,
    ) -> Self {
        self.virtual_geometry_indirect_draw_count = stats.draw_count;
        self.virtual_geometry_indirect_buffer_count = stats.buffer_count;
        self.virtual_geometry_indirect_args_count = stats.args_count;
        self.virtual_geometry_indirect_segment_count = stats.segment_count;
        self
    }

    pub(crate) fn with_virtual_geometry_execution_stats(
        mut self,
        stats: PreparedMeshVirtualGeometryExecutionStats,
    ) -> Self {
        self.virtual_geometry_execution_draw_count = stats.draw_count;
        self.virtual_geometry_execution_segment_count = stats.segment_count;
        self.virtual_geometry_execution_page_count = stats.page_count;
        self.virtual_geometry_execution_resident_segment_count = stats.resident_segment_count;
        self.virtual_geometry_execution_pending_segment_count = stats.pending_segment_count;
        self.virtual_geometry_execution_missing_segment_count = stats.missing_segment_count;
        self.virtual_geometry_execution_repeated_draw_count = stats.repeated_draw_count;
        self
    }

    pub(crate) fn with_mesh_draw_replay_stats(mut self, replay_stats: MeshDrawReplayStats) -> Self {
        self.indirect_count_draw_call_count = replay_stats.indirect_count_draw_call_count as usize;
        self.fixed_multi_draw_call_count = replay_stats.fixed_multi_draw_call_count as usize;
        self.per_draw_indirect_draw_call_count =
            replay_stats.per_draw_indirect_draw_call_count as usize;
        self.direct_draw_call_count = replay_stats.direct_draw_call_count as usize;
        self.state_change_count = replay_stats.state_change_count as usize;
        self.bind_skip_count = replay_stats.bind_skip_count as usize;
        self
    }
}
