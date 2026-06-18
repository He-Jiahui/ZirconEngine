use crate::core::framework::render::RenderStats;

use super::{record_count, DiagnosticStore};
pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.mesh.queue.draw_count",
        frame_index,
        stats.last_mesh_draw_count,
        &["render", "mesh", "queue"],
    );
    record_count(
        store,
        "render.mesh.queue.opaque_draw_count",
        frame_index,
        stats.last_mesh_opaque_draw_count,
        &["render", "mesh", "queue"],
    );
    record_count(
        store,
        "render.mesh.queue.alpha_mask_draw_count",
        frame_index,
        stats.last_mesh_alpha_mask_draw_count,
        &["render", "mesh", "queue"],
    );
    record_count(
        store,
        "render.mesh.queue.transparent_draw_count",
        frame_index,
        stats.last_mesh_transparent_draw_count,
        &["render", "mesh", "queue"],
    );
    record_count(
        store,
        "render.mesh.queue.early_z_draw_count",
        frame_index,
        stats.last_mesh_early_z_draw_count,
        &["render", "mesh", "queue", "early_z"],
    );
    record_count(
        store,
        "render.mesh.queue.shadow_caster_draw_count",
        frame_index,
        stats.last_mesh_shadow_caster_draw_count,
        &["render", "mesh", "queue", "shadow"],
    );
    record_count(
        store,
        "render.mesh.queue.alpha_mask_shadow_caster_draw_count",
        frame_index,
        stats.last_mesh_alpha_mask_shadow_caster_draw_count,
        &["render", "mesh", "queue", "shadow", "alpha_mask"],
    );
    record_count(
        store,
        "render.mesh.queue.prepared_geometry_draw_count",
        frame_index,
        stats.last_mesh_prepared_geometry_draw_count,
        &["render", "mesh", "queue"],
    );
    record_count(
        store,
        "render.mesh.queue.dynamic_geometry_draw_count",
        frame_index,
        stats.last_mesh_dynamic_geometry_draw_count,
        &["render", "mesh", "queue"],
    );
    record_count(
        store,
        "render.mesh.queue.skinned_draw_count",
        frame_index,
        stats.last_mesh_skinned_draw_count,
        &["render", "mesh", "queue", "skinned"],
    );
    record_count(
        store,
        "render.mesh.queue.skinned_palette_upload_count",
        frame_index,
        stats.last_mesh_skinned_palette_upload_count,
        &["render", "mesh", "queue", "skinned", "palette"],
    );
    record_count(
        store,
        "render.mesh.queue.skinned_previous_palette_upload_count",
        frame_index,
        stats.last_mesh_skinned_previous_palette_upload_count,
        &["render", "mesh", "queue", "skinned", "palette", "previous"],
    );
    record_count(
        store,
        "render.mesh.queue.skinned_gpu_source_candidate_count",
        frame_index,
        stats.last_mesh_skinned_gpu_source_candidate_count,
        &["render", "mesh", "queue", "skinned", "gpu_source"],
    );
    record_count(
        store,
        "render.mesh.queue.skinned_gpu_cpu_morphed_source_candidate_count",
        frame_index,
        stats.last_mesh_skinned_gpu_cpu_morphed_source_candidate_count,
        &[
            "render",
            "mesh",
            "queue",
            "skinned",
            "gpu_source",
            "cpu_morphed",
        ],
    );
    record_count(
        store,
        "render.mesh.queue.skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count",
        frame_index,
        stats.last_mesh_skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count,
        &[
            "render",
            "mesh",
            "queue",
            "skinned",
            "gpu_source",
            "cpu_morphed",
            "previous_shape_missing",
            "velocity",
        ],
    );
    record_count(
        store,
        "render.mesh.queue.skinned_gpu_skinning_draw_count",
        frame_index,
        stats.last_mesh_skinned_gpu_skinning_draw_count,
        &["render", "mesh", "queue", "skinned", "gpu_skinning"],
    );
    record_count(
        store,
        "render.mesh.queue.skinned_gpu_velocity_draw_count",
        frame_index,
        stats.last_mesh_skinned_gpu_velocity_draw_count,
        &[
            "render",
            "mesh",
            "queue",
            "skinned",
            "gpu_skinning",
            "velocity",
        ],
    );
    record_count(
        store,
        "render.mesh.queue.indirect_draw_count",
        frame_index,
        stats.last_mesh_indirect_draw_count,
        &["render", "mesh", "queue", "indirect"],
    );
    record_count(
        store,
        "render.mesh.queue.lod_draw_count",
        frame_index,
        stats.last_mesh_lod_draw_count,
        &["render", "mesh", "queue", "lod"],
    );
    record_count(
        store,
        "render.mesh.queue.previous_velocity_transform_draw_count",
        frame_index,
        stats.last_mesh_previous_velocity_transform_draw_count,
        &["render", "mesh", "queue", "velocity", "previous"],
    );
    record_count(
        store,
        "render.mesh.queue.missing_velocity_transform_draw_count",
        frame_index,
        stats.last_mesh_missing_velocity_transform_draw_count,
        &["render", "mesh", "queue", "velocity", "missing"],
    );
    record_count(
        store,
        "render.mesh.queue.taa_reactive_mask_command_count",
        frame_index,
        stats.last_mesh_taa_reactive_mask_command_count,
        &["render", "mesh", "queue", "taa", "reactive_mask"],
    );
    record_count(
        store,
        "render.mesh.queue.static_batch_candidate_group_count",
        frame_index,
        stats.last_mesh_static_batch_candidate_group_count,
        &["render", "mesh", "queue", "batch"],
    );
    record_count(
        store,
        "render.mesh.queue.static_batch_candidate_draw_count",
        frame_index,
        stats.last_mesh_static_batch_candidate_draw_count,
        &["render", "mesh", "queue", "batch"],
    );
    record_count(
        store,
        "render.mesh.queue.dynamic_batch_candidate_group_count",
        frame_index,
        stats.last_mesh_dynamic_batch_candidate_group_count,
        &["render", "mesh", "queue", "batch"],
    );
    record_count(
        store,
        "render.mesh.queue.dynamic_batch_candidate_draw_count",
        frame_index,
        stats.last_mesh_dynamic_batch_candidate_draw_count,
        &["render", "mesh", "queue", "batch"],
    );
    record_count(
        store,
        "render.mesh.queue.gpu_instancing_candidate_group_count",
        frame_index,
        stats.last_mesh_gpu_instancing_candidate_group_count,
        &["render", "mesh", "queue", "instancing"],
    );
    record_count(
        store,
        "render.mesh.queue.gpu_instancing_candidate_draw_count",
        frame_index,
        stats.last_mesh_gpu_instancing_candidate_draw_count,
        &["render", "mesh", "queue", "instancing"],
    );
    record_count(
        store,
        "render.mesh.queue.command_count",
        frame_index,
        stats.last_mesh_command_count,
        &["render", "mesh", "queue", "command"],
    );
    record_count(
        store,
        "render.mesh.queue.cached_command_hit_count",
        frame_index,
        stats.last_mesh_cached_command_hit_count,
        &["render", "mesh", "queue", "command", "cache"],
    );
    record_count(
        store,
        "render.mesh.queue.command_rebuild_count",
        frame_index,
        stats.last_mesh_command_rebuild_count,
        &["render", "mesh", "queue", "command", "cache", "rebuild"],
    );
    record_count(
        store,
        "render.mesh.queue.dynamic_command_count",
        frame_index,
        stats.last_mesh_dynamic_command_count,
        &["render", "mesh", "queue", "command", "dynamic"],
    );
    record_count(
        store,
        "render.mesh.queue.command_cache_miss_count",
        frame_index,
        stats.last_mesh_command_cache_miss_count,
        &["render", "mesh", "queue", "command", "cache", "miss"],
    );
    record_count(
        store,
        "render.mesh.queue.command_cache_invalidated_transform_count",
        frame_index,
        stats.last_mesh_command_cache_invalidated_transform_count,
        &[
            "render",
            "mesh",
            "queue",
            "command",
            "cache",
            "invalidated",
            "transform",
        ],
    );
    record_count(
        store,
        "render.mesh.queue.command_cache_invalidated_geometry_count",
        frame_index,
        stats.last_mesh_command_cache_invalidated_geometry_count,
        &[
            "render",
            "mesh",
            "queue",
            "command",
            "cache",
            "invalidated",
            "geometry",
        ],
    );
    record_count(
        store,
        "render.mesh.queue.command_cache_invalidated_material_count",
        frame_index,
        stats.last_mesh_command_cache_invalidated_material_count,
        &[
            "render",
            "mesh",
            "queue",
            "command",
            "cache",
            "invalidated",
            "material",
        ],
    );
    record_count(
        store,
        "render.mesh.queue.replay_state_change_count",
        frame_index,
        stats.last_mesh_replay_state_change_count,
        &["render", "mesh", "queue", "command", "replay", "state"],
    );
    record_count(
        store,
        "render.mesh.queue.replay_bind_skip_count",
        frame_index,
        stats.last_mesh_replay_bind_skip_count,
        &["render", "mesh", "queue", "command", "replay", "bind"],
    );
    record_count(
        store,
        "render.mesh.queue.indirect_batch_count",
        frame_index,
        stats.last_indirect_batch_count,
        &["render", "mesh", "queue", "indirect", "batch"],
    );
    record_count(
        store,
        "render.mesh.queue.indirect_batched_draw_count",
        frame_index,
        stats.last_indirect_batched_draw_count,
        &["render", "mesh", "queue", "indirect", "batch"],
    );
    record_count(
        store,
        "render.mesh.queue.indirect_fallback_draw_count",
        frame_index,
        stats.last_indirect_fallback_draw_count,
        &["render", "mesh", "queue", "indirect", "fallback"],
    );
    record_count(
        store,
        "render.mesh.queue.indirect_args_count",
        frame_index,
        stats.last_indirect_args_count,
        &["render", "mesh", "queue", "indirect", "args"],
    );
}
