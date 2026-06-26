use crate::core::diagnostics::DiagnosticStore;
use crate::core::framework::render::{RenderGpuSceneUploadPath, RenderStats};

use super::super::record;
use super::assert_series;

#[test]
fn render_product_diagnostics_record_skinned_mesh_queue_count() {
    let mut store = DiagnosticStore::default();
    let stats = RenderStats {
        submitted_frames: 12,
        last_mesh_skinned_draw_count: 3,
        last_mesh_skinned_palette_upload_count: 2,
        last_mesh_skinned_previous_palette_upload_count: 1,
        last_mesh_skinned_gpu_source_candidate_count: 1,
        last_mesh_skinned_gpu_cpu_morphed_source_candidate_count: 1,
        last_mesh_skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count: 1,
        last_mesh_skinned_gpu_skinning_draw_count: 1,
        last_mesh_skinned_gpu_velocity_draw_count: 1,
        ..RenderStats::default()
    };

    record(&mut store, &stats);

    assert_series(&store, "render.mesh.queue.skinned_draw_count", 3.0, "count");
    assert_series(
        &store,
        "render.mesh.queue.skinned_palette_upload_count",
        2.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.skinned_previous_palette_upload_count",
        1.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.skinned_gpu_source_candidate_count",
        1.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.skinned_gpu_cpu_morphed_source_candidate_count",
        1.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count",
        1.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.skinned_gpu_skinning_draw_count",
        1.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.skinned_gpu_velocity_draw_count",
        1.0,
        "count",
    );
}

#[test]
fn render_product_diagnostics_record_gpu_scene_upload_stats() {
    let mut store = DiagnosticStore::default();
    let stats = RenderStats {
        submitted_frames: 12,
        last_gpu_scene_primitive_count: 5,
        last_gpu_scene_instance_count: 7,
        last_gpu_scene_dirty_entry_count: 3,
        last_gpu_scene_uploaded_bytes: 128,
        last_gpu_scene_upload_path: RenderGpuSceneUploadPath::DirectQueueWrite,
        last_gpu_scene_free_span_count: 2,
        last_gpu_scene_primitive_upload_range_count: 1,
        last_gpu_scene_instance_upload_range_count: 4,
        ..RenderStats::default()
    };

    record(&mut store, &stats);

    assert_series(&store, "render.gpu_scene.primitive_count", 5.0, "count");
    assert_series(&store, "render.gpu_scene.instance_count", 7.0, "count");
    assert_series(&store, "render.gpu_scene.dirty_entry_count", 3.0, "count");
    assert_series(&store, "render.gpu_scene.uploaded_bytes", 128.0, "bytes");
    assert_series(
        &store,
        "render.gpu_scene.upload_path.direct_queue_write",
        1.0,
        "bool",
    );
    assert_series(&store, "render.gpu_scene.free_span_count", 2.0, "count");
    assert_series(
        &store,
        "render.gpu_scene.primitive_upload_range_count",
        1.0,
        "count",
    );
    assert_series(
        &store,
        "render.gpu_scene.instance_upload_range_count",
        4.0,
        "count",
    );
}

#[test]
fn render_product_diagnostics_record_mesh_indirect_batch_stats() {
    let mut store = DiagnosticStore::default();
    let stats = RenderStats {
        submitted_frames: 12,
        last_indirect_batch_count: 2,
        last_indirect_batched_draw_count: 5,
        last_indirect_fallback_draw_count: 4,
        last_indirect_args_count: 5,
        ..RenderStats::default()
    };

    record(&mut store, &stats);

    assert_series(
        &store,
        "render.mesh.queue.indirect_batch_count",
        2.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.indirect_batched_draw_count",
        5.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.indirect_fallback_draw_count",
        4.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.indirect_args_count",
        5.0,
        "count",
    );
}

#[test]
fn render_product_diagnostics_record_mesh_lod_queue_count() {
    let mut store = DiagnosticStore::default();
    let stats = RenderStats {
        submitted_frames: 12,
        last_mesh_lod_draw_count: 4,
        ..RenderStats::default()
    };

    record(&mut store, &stats);

    assert_series(&store, "render.mesh.queue.lod_draw_count", 4.0, "count");
}

#[test]
fn render_product_diagnostics_record_taa_reactive_mask_queue_count() {
    let mut store = DiagnosticStore::default();
    let stats = RenderStats {
        submitted_frames: 12,
        last_mesh_taa_reactive_mask_command_count: 3,
        ..RenderStats::default()
    };

    record(&mut store, &stats);

    assert_series(
        &store,
        "render.mesh.queue.taa_reactive_mask_command_count",
        3.0,
        "count",
    );
}

#[test]
fn render_product_diagnostics_record_mesh_command_cache_counts() {
    let mut store = DiagnosticStore::default();
    let stats = RenderStats {
        submitted_frames: 12,
        last_mesh_command_count: 9,
        last_mesh_cached_command_hit_count: 4,
        last_mesh_command_rebuild_count: 5,
        last_mesh_dynamic_command_count: 2,
        last_mesh_pending_static_command_cache_draw_candidate_count: 3,
        last_mesh_pending_static_command_cache_phase_candidate_count: 7,
        last_mesh_pending_static_command_cache_depth_prepass_candidate_count: 3,
        last_mesh_pending_static_command_cache_shadow_candidate_count: 2,
        last_mesh_pending_static_command_cache_opaque_candidate_count: 1,
        last_mesh_pending_static_command_cache_alpha_mask_candidate_count: 1,
        last_mesh_pre_mesh_draw_static_command_cache_skipped_draw_count: 2,
        last_mesh_pre_mesh_draw_static_command_cache_skipped_phase_count: 5,
        last_mesh_pre_mesh_draw_static_command_cache_visibility_pruned_draw_count: 1,
        last_mesh_pre_mesh_draw_static_command_cache_residual_material_phase_draw_count: 3,
        last_mesh_pre_mesh_draw_static_command_cache_residual_rebuild_input_missing_draw_count: 4,
        last_mesh_pre_mesh_draw_static_command_cache_residual_rebuild_rejected_draw_count: 6,
        last_mesh_command_cache_miss_count: 1,
        last_mesh_command_cache_invalidated_transform_count: 0,
        last_mesh_command_cache_invalidated_geometry_count: 1,
        last_mesh_command_cache_invalidated_material_count: 2,
        last_mesh_replay_state_change_count: 6,
        last_mesh_replay_bind_skip_count: 7,
        ..RenderStats::default()
    };

    record(&mut store, &stats);

    assert_series(&store, "render.mesh.queue.command_count", 9.0, "count");
    assert_series(
        &store,
        "render.mesh.queue.cached_command_hit_count",
        4.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.command_rebuild_count",
        5.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.dynamic_command_count",
        2.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.pending_static_command_cache.draw_candidate_count",
        3.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.pending_static_command_cache.phase_candidate_count",
        7.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.pending_static_command_cache.depth_prepass_candidate_count",
        3.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.pending_static_command_cache.shadow_candidate_count",
        2.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.pending_static_command_cache.opaque_candidate_count",
        1.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.pending_static_command_cache.alpha_mask_candidate_count",
        1.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.pre_mesh_draw_static_command_cache.skipped_draw_count",
        2.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.pre_mesh_draw_static_command_cache.skipped_phase_count",
        5.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.pre_mesh_draw_static_command_cache.visibility_pruned_draw_count",
        1.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.pre_mesh_draw_static_command_cache.residual_material_phase_draw_count",
        3.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.pre_mesh_draw_static_command_cache.residual_rebuild_input_missing_draw_count",
        4.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.pre_mesh_draw_static_command_cache.residual_rebuild_rejected_draw_count",
        6.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.command_cache_miss_count",
        1.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.command_cache_invalidated_transform_count",
        0.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.command_cache_invalidated_geometry_count",
        1.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.command_cache_invalidated_material_count",
        2.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.replay_state_change_count",
        6.0,
        "count",
    );
    assert_series(
        &store,
        "render.mesh.queue.replay_bind_skip_count",
        7.0,
        "count",
    );
}
