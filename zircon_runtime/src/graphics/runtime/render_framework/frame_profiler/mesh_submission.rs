use crate::core::framework::render::{RenderMeshSubmissionProfile, RenderStats};

use super::saturating_u32;

pub(super) fn mesh_submission_profile(stats: &RenderStats) -> RenderMeshSubmissionProfile {
    RenderMeshSubmissionProfile {
        draw_count: saturating_u32(stats.last_mesh_draw_count),
        command_count: saturating_u32(stats.last_mesh_command_count),
        opaque_command_count: saturating_u32(stats.last_mesh_opaque_command_count),
        advanced_pbr_opaque_command_count: saturating_u32(
            stats.last_mesh_advanced_pbr_opaque_command_count,
        ),
        cached_command_hit_count: saturating_u32(stats.last_mesh_cached_command_hit_count),
        command_rebuild_count: saturating_u32(stats.last_mesh_command_rebuild_count),
        dynamic_command_count: saturating_u32(stats.last_mesh_dynamic_command_count),
        static_command_cache_skipped_draw_count: saturating_u32(
            stats.last_mesh_pre_mesh_draw_static_command_cache_skipped_draw_count,
        ),
        static_command_cache_visibility_pruned_draw_count: saturating_u32(
            stats.last_mesh_pre_mesh_draw_static_command_cache_visibility_pruned_draw_count,
        ),
        indirect_batch_count: saturating_u32(stats.last_indirect_batch_count),
        indirect_batched_draw_count: saturating_u32(stats.last_indirect_batched_draw_count),
        indirect_fallback_draw_count: saturating_u32(stats.last_indirect_fallback_draw_count),
        indirect_workspace_uploaded_bytes: stats.last_indirect_workspace_uploaded_byte_count,
        replay_state_change_count: saturating_u32(stats.last_mesh_replay_state_change_count),
        replay_bind_skip_count: saturating_u32(stats.last_mesh_replay_bind_skip_count),
        material_bind_group_set_count: saturating_u32(
            stats.last_mesh_replay_material_bind_group_set_count,
        ),
        material_bind_group_skip_count: saturating_u32(
            stats.last_mesh_replay_material_bind_group_skip_count,
        ),
    }
}

#[cfg(test)]
#[path = "tests/mesh_submission.rs"]
mod tests;
