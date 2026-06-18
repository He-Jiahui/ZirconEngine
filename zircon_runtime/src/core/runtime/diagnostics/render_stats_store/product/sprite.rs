use crate::core::framework::render::RenderStats;

use super::{record_count, DiagnosticStore};
pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
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
        "render.sprite.queue.image_slice_count",
        frame_index,
        stats.last_sprite_image_slice_count,
        &["render", "sprite", "queue", "image_slice"],
    );
    record_count(
        store,
        "render.sprite.queue.expanded_image_slice_count",
        frame_index,
        stats.last_sprite_expanded_image_slice_count,
        &["render", "sprite", "queue", "image_slice", "expanded"],
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
