use crate::core::diagnostics::RuntimeDiagnosticsSnapshot;

use super::support::{
    assert_light_family_series, assert_render_bool_series, assert_render_count_series,
    assert_series_current,
};

pub(super) fn assert_post_process_material_mesh(snapshot: &RuntimeDiagnosticsSnapshot) {
    assert_render_bool_series(
        &snapshot.store,
        "render.anti_alias.requested_requires_history",
        true,
        &["anti_alias", "requested", "history"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.anti_alias.effective_post_process",
        true,
        &["anti_alias", "effective"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.anti_alias.fallback.active",
        true,
        &["anti_alias", "fallback"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.anti_alias.fallback.missing_history",
        true,
        &["anti_alias", "fallback", "history"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.anti_alias.normalization.graph_sample_count",
        true,
        &["anti_alias", "normalization", "graph", "msaa"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.anti_alias.normalization.taa_msaa_conflict",
        true,
        &["anti_alias", "normalization", "taa", "msaa"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.anti_alias.normalization.count",
        1.0,
        &["anti_alias", "normalization"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.particle.gpu.alive_count",
        31.0,
        &["particle", "gpu"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.particle.velocity.missing_sprite_count",
        5.0,
        &["particle", "velocity", "missing"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.particle.gpu.indirect_instance_count",
        29.0,
        &["particle", "gpu", "indirect"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.graph.node_count",
        5.0,
        &["post_process", "graph"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.graph.skipped_node_count",
        1.0,
        &["post_process", "graph"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.graph.executed_node_count",
        3.0,
        &["post_process", "graph"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.post_process.graph.output_transfer_present",
        true,
        &["post_process", "graph", "output_transfer"],
    );
    assert_series_current(
        &snapshot.store,
        "render.post_process.effect_stack.enabled",
        1.0,
        "bool",
    );
    assert_series_current(
        &snapshot.store,
        "render.post_process.effect_stack.active_family_count",
        3.0,
        "count",
    );
    assert_series_current(
        &snapshot.store,
        "render.post_process.effect_stack.approximated_family_count",
        2.0,
        "count",
    );
    assert_series_current(
        &snapshot.store,
        "render.post_process.effect_stack.missing_resource_count",
        1.0,
        "count",
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.lut.request_count",
        1.0,
        &["post_process", "lut"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.lut.ready_count",
        0.0,
        &["post_process", "lut", "ready"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.lut.fallback_count",
        1.0,
        &["post_process", "lut", "fallback"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.lut.texture_2d_strip_ready_count",
        0.0,
        &["post_process", "lut", "texture_2d_strip", "ready"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.lut.texture_3d_request_count",
        1.0,
        &["post_process", "lut", "texture_3d"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.post_process.lut.unsupported_shape_count",
        0.0,
        &["post_process", "lut", "unsupported_shape"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.post_process.motion_vector.camera.not_requested",
        false,
        &["post_process", "motion_vector", "camera"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.post_process.motion_vector.camera.missing_previous_camera",
        false,
        &["post_process", "motion_vector", "camera"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.post_process.motion_vector.camera.cut_or_invalid",
        false,
        &["post_process", "motion_vector", "camera"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.post_process.motion_vector.camera.ready",
        true,
        &["post_process", "motion_vector", "camera", "ready"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.material.count",
        13.0,
        &["material"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.material.ready_count",
        10.0,
        &["material"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.material.fallback_count",
        2.0,
        &["material", "fallback"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.material.validation_error_count",
        1.0,
        &["material", "validation"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.material.diagnostic_count",
        4.0,
        &["material", "diagnostic"],
    );
    assert_light_family_series(&snapshot.store, "directional", 3.0, 3.0, 0.0);
    assert_light_family_series(&snapshot.store, "point", 4.0, 4.0, 0.0);
    assert_light_family_series(&snapshot.store, "spot", 5.0, 5.0, 0.0);
    assert_light_family_series(&snapshot.store, "ambient", 2.0, 2.0, 0.0);
    assert_light_family_series(&snapshot.store, "rect", 1.0, 0.0, 1.0);
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.draw_count",
        12.0,
        &["mesh", "queue"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.opaque_draw_count",
        6.0,
        &["mesh", "queue"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.alpha_mask_draw_count",
        2.0,
        &["mesh", "queue"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.transparent_draw_count",
        4.0,
        &["mesh", "queue"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.early_z_draw_count",
        8.0,
        &["mesh", "queue", "early_z"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.shadow_caster_draw_count",
        8.0,
        &["mesh", "queue", "shadow"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.alpha_mask_shadow_caster_draw_count",
        2.0,
        &["mesh", "queue", "shadow", "alpha_mask"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.prepared_geometry_draw_count",
        5.0,
        &["mesh", "queue"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.dynamic_geometry_draw_count",
        7.0,
        &["mesh", "queue"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.gpu_morphed_source_draw_count",
        2.0,
        &["mesh", "queue", "morph", "gpu_source"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.gpu_skinned_morphed_source_draw_count",
        1.0,
        &["mesh", "queue", "morph", "gpu_source", "skinned"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.skinned_draw_count",
        3.0,
        &["mesh", "queue", "skinned"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.skinned_gpu_source_candidate_count",
        1.0,
        &["mesh", "queue", "skinned", "gpu_source"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.skinned_gpu_cpu_morphed_source_candidate_count",
        1.0,
        &["mesh", "queue", "skinned", "gpu_source", "cpu_morphed"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count",
        1.0,
        &[
            "mesh",
            "queue",
            "skinned",
            "gpu_source",
            "cpu_morphed",
            "previous_shape_missing",
            "velocity",
        ],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.skinned_gpu_skinning_draw_count",
        1.0,
        &["mesh", "queue", "skinned", "gpu_skinning"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.indirect_draw_count",
        3.0,
        &["mesh", "queue", "indirect"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.lod_draw_count",
        2.0,
        &["mesh", "queue", "lod"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.static_batch_candidate_group_count",
        2.0,
        &["mesh", "queue", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.static_batch_candidate_draw_count",
        5.0,
        &["mesh", "queue", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.dynamic_batch_candidate_group_count",
        3.0,
        &["mesh", "queue", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.dynamic_batch_candidate_draw_count",
        6.0,
        &["mesh", "queue", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.gpu_instancing_candidate_group_count",
        4.0,
        &["mesh", "queue", "instancing"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.gpu_instancing_candidate_draw_count",
        9.0,
        &["mesh", "queue", "instancing"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.indirect_batch_count",
        2.0,
        &["mesh", "queue", "indirect", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.indirect_batched_draw_count",
        5.0,
        &["mesh", "queue", "indirect", "batch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.indirect_fallback_draw_count",
        4.0,
        &["mesh", "queue", "indirect", "fallback"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.mesh.queue.indirect_args_count",
        5.0,
        &["mesh", "queue", "indirect", "args"],
    );
}
