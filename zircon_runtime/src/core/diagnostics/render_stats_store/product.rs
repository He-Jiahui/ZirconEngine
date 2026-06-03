use crate::core::framework::render::RenderStats;

use super::{record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    record_material(store, stats);
    record_light(store, stats);
    record_mesh_queue(store, stats);
    record_sprite(store, stats);
    record_effect_stack(store, stats);
    record_ui(store, stats);
}

fn record_effect_stack(store: &mut DiagnosticStore, stats: &RenderStats) {
    let report = &stats.last_post_process_effect_stack_report;
    let frame_index = stats.submitted_frames;
    store.record(
        "render.post_process.effect_stack.enabled",
        frame_index,
        u8::from(report.enabled) as f64,
        Some("bool"),
        ["render", "post_process", "effect_stack"],
    );
    store.record(
        "render.post_process.effect_stack.active_family_count",
        frame_index,
        report.active_family_count as f64,
        Some("count"),
        ["render", "post_process", "effect_stack"],
    );
    store.record(
        "render.post_process.effect_stack.approximated_family_count",
        frame_index,
        report.approximated_family_count as f64,
        Some("count"),
        ["render", "post_process", "effect_stack"],
    );
    store.record(
        "render.post_process.effect_stack.missing_resource_count",
        frame_index,
        report.missing_resource_count as f64,
        Some("count"),
        ["render", "post_process", "effect_stack"],
    );
    record_count(
        store,
        "render.post_process.lut.request_count",
        frame_index,
        stats.last_post_process_lut_request_count,
        &["render", "post_process", "lut"],
    );
    record_count(
        store,
        "render.post_process.lut.ready_count",
        frame_index,
        stats.last_post_process_lut_ready_count,
        &["render", "post_process", "lut", "ready"],
    );
    record_count(
        store,
        "render.post_process.lut.fallback_count",
        frame_index,
        stats.last_post_process_lut_fallback_count,
        &["render", "post_process", "lut", "fallback"],
    );
    record_count(
        store,
        "render.post_process.lut.texture_2d_strip_ready_count",
        frame_index,
        stats.last_post_process_lut_2d_strip_ready_count,
        &["render", "post_process", "lut", "texture_2d_strip", "ready"],
    );
    record_count(
        store,
        "render.post_process.lut.texture_3d_request_count",
        frame_index,
        stats.last_post_process_lut_3d_request_count,
        &["render", "post_process", "lut", "texture_3d"],
    );
    record_count(
        store,
        "render.post_process.lut.unsupported_shape_count",
        frame_index,
        stats.last_post_process_lut_unsupported_shape_count,
        &["render", "post_process", "lut", "unsupported_shape"],
    );
}

fn record_material(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.material.count",
        frame_index,
        stats.last_material_count,
        &["render", "material"],
    );
    record_count(
        store,
        "render.material.ready_count",
        frame_index,
        stats.last_material_ready_count,
        &["render", "material"],
    );
    record_count(
        store,
        "render.material.fallback_count",
        frame_index,
        stats.last_material_fallback_count,
        &["render", "material", "fallback"],
    );
    record_count(
        store,
        "render.material.validation_error_count",
        frame_index,
        stats.last_material_validation_error_count,
        &["render", "material", "validation"],
    );
    record_count(
        store,
        "render.material.diagnostic_count",
        frame_index,
        stats.last_material_diagnostic_count,
        &["render", "material", "diagnostic"],
    );
}

fn record_light(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_light_family(
        store,
        frame_index,
        "directional",
        stats.last_directional_light_count,
        stats.last_directional_light_ready_count,
        stats.last_directional_light_degraded_count,
        (
            "render.light.directional.count",
            "render.light.directional.ready_count",
            "render.light.directional.degraded_count",
        ),
    );
    record_light_family(
        store,
        frame_index,
        "point",
        stats.last_point_light_count,
        stats.last_point_light_ready_count,
        stats.last_point_light_degraded_count,
        (
            "render.light.point.count",
            "render.light.point.ready_count",
            "render.light.point.degraded_count",
        ),
    );
    record_light_family(
        store,
        frame_index,
        "spot",
        stats.last_spot_light_count,
        stats.last_spot_light_ready_count,
        stats.last_spot_light_degraded_count,
        (
            "render.light.spot.count",
            "render.light.spot.ready_count",
            "render.light.spot.degraded_count",
        ),
    );
    record_light_family(
        store,
        frame_index,
        "ambient",
        stats.last_ambient_light_count,
        stats.last_ambient_light_ready_count,
        stats.last_ambient_light_degraded_count,
        (
            "render.light.ambient.count",
            "render.light.ambient.ready_count",
            "render.light.ambient.degraded_count",
        ),
    );
    record_light_family(
        store,
        frame_index,
        "rect",
        stats.last_rect_light_count,
        stats.last_rect_light_ready_count,
        stats.last_rect_light_degraded_count,
        (
            "render.light.rect.count",
            "render.light.rect.ready_count",
            "render.light.rect.degraded_count",
        ),
    );
}

fn record_mesh_queue(store: &mut DiagnosticStore, stats: &RenderStats) {
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
        "render.mesh.queue.indirect_draw_count",
        frame_index,
        stats.last_mesh_indirect_draw_count,
        &["render", "mesh", "queue", "indirect"],
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
}

fn record_sprite(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.sprite.count",
        frame_index,
        stats.last_sprite_count,
        &["render", "sprite"],
    );
    record_count(
        store,
        "render.sprite.ready_count",
        frame_index,
        stats.last_sprite_ready_count,
        &["render", "sprite"],
    );
    record_count(
        store,
        "render.sprite.texture_fallback_count",
        frame_index,
        stats.last_sprite_texture_fallback_count,
        &["render", "sprite", "fallback"],
    );
    record_count(
        store,
        "render.sprite.graph_executed_pass_count",
        frame_index,
        stats.last_sprite_graph_executed_pass_count,
        &["render", "sprite", "graph"],
    );
    record_count(
        store,
        "render.sprite.queue.draw_batch_count",
        frame_index,
        stats.last_sprite_draw_batch_count,
        &["render", "sprite", "queue", "batch"],
    );
    record_count(
        store,
        "render.sprite.queue.batched_sprite_count",
        frame_index,
        stats.last_sprite_batched_sprite_count,
        &["render", "sprite", "queue", "batch"],
    );
    record_count(
        store,
        "render.sprite.queue.vertex_count",
        frame_index,
        stats.last_sprite_vertex_count,
        &["render", "sprite", "queue"],
    );
    record_count(
        store,
        "render.sprite.queue.opaque_draw_batch_count",
        frame_index,
        stats.last_sprite_opaque_draw_batch_count,
        &["render", "sprite", "queue", "batch"],
    );
    record_count(
        store,
        "render.sprite.queue.alpha_mask_draw_batch_count",
        frame_index,
        stats.last_sprite_alpha_mask_draw_batch_count,
        &["render", "sprite", "queue", "batch"],
    );
    record_count(
        store,
        "render.sprite.queue.transparent_draw_batch_count",
        frame_index,
        stats.last_sprite_transparent_draw_batch_count,
        &["render", "sprite", "queue", "batch"],
    );
}

fn record_ui(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.ui.command_count",
        frame_index,
        stats.last_ui_command_count,
        &["render", "ui"],
    );
    record_count(
        store,
        "render.ui.quad_count",
        frame_index,
        stats.last_ui_quad_count,
        &["render", "ui"],
    );
    record_count(
        store,
        "render.ui.text_payload_count",
        frame_index,
        stats.last_ui_text_payload_count,
        &["render", "ui", "text"],
    );
    record_count(
        store,
        "render.ui.image_payload_count",
        frame_index,
        stats.last_ui_image_payload_count,
        &["render", "ui", "image"],
    );
    record_count(
        store,
        "render.ui.clipped_command_count",
        frame_index,
        stats.last_ui_clipped_command_count,
        &["render", "ui", "clip"],
    );
    record_count(
        store,
        "render.ui.graph_executed_pass_count",
        frame_index,
        stats.last_ui_graph_executed_pass_count,
        &["render", "ui", "graph"],
    );
}

fn record_light_family(
    store: &mut DiagnosticStore,
    frame_index: u64,
    family_tag: &'static str,
    count: usize,
    ready_count: usize,
    degraded_count: usize,
    paths: (&'static str, &'static str, &'static str),
) {
    record_count(
        store,
        paths.0,
        frame_index,
        count,
        &["render", "light", family_tag],
    );
    record_count(
        store,
        paths.1,
        frame_index,
        ready_count,
        &["render", "light", family_tag, "ready"],
    );
    record_count(
        store,
        paths.2,
        frame_index,
        degraded_count,
        &["render", "light", family_tag, "degraded"],
    );
}
