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
mod tests {
    use crate::core::framework::render::{RenderMeshSubmissionProfile, RenderStats};

    use super::mesh_submission_profile;

    #[test]
    fn snapshot_keeps_mesh_cache_indirect_and_replay_metrics_together() {
        let stats = RenderStats {
            last_mesh_draw_count: 17,
            last_mesh_command_count: 9,
            last_mesh_opaque_command_count: 8,
            last_mesh_advanced_pbr_opaque_command_count: 1,
            last_mesh_cached_command_hit_count: 5,
            last_mesh_command_rebuild_count: 3,
            last_mesh_dynamic_command_count: 2,
            last_mesh_pre_mesh_draw_static_command_cache_skipped_draw_count: 7,
            last_mesh_pre_mesh_draw_static_command_cache_visibility_pruned_draw_count: 4,
            last_indirect_batch_count: 6,
            last_indirect_batched_draw_count: 11,
            last_indirect_fallback_draw_count: 1,
            last_indirect_workspace_uploaded_byte_count: 4_096,
            last_mesh_replay_state_change_count: 13,
            last_mesh_replay_bind_skip_count: 8,
            last_mesh_replay_material_bind_group_set_count: 12,
            last_mesh_replay_material_bind_group_skip_count: 10,
            ..RenderStats::default()
        };

        assert_eq!(
            mesh_submission_profile(&stats),
            RenderMeshSubmissionProfile {
                draw_count: 17,
                command_count: 9,
                opaque_command_count: 8,
                advanced_pbr_opaque_command_count: 1,
                cached_command_hit_count: 5,
                command_rebuild_count: 3,
                dynamic_command_count: 2,
                static_command_cache_skipped_draw_count: 7,
                static_command_cache_visibility_pruned_draw_count: 4,
                indirect_batch_count: 6,
                indirect_batched_draw_count: 11,
                indirect_fallback_draw_count: 1,
                indirect_workspace_uploaded_bytes: 4_096,
                replay_state_change_count: 13,
                replay_bind_skip_count: 8,
                material_bind_group_set_count: 12,
                material_bind_group_skip_count: 10,
            }
        );
    }
}
