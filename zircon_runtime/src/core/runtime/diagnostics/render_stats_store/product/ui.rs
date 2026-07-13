use crate::core::framework::render::RenderStats;

use super::{record_count, DiagnosticStore};
pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
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
        "render.ui.text.glyph_count",
        frame_index,
        stats.last_ui_text_glyph_count,
        &["render", "ui", "text"],
    );
    record_count(
        store,
        "render.ui.text.unmapped_glyph_count",
        frame_index,
        stats.last_ui_text_unmapped_glyph_count,
        &["render", "ui", "text"],
    );
    record_count(
        store,
        "render.ui.text.raster.visible_glyph_count",
        frame_index,
        stats.last_ui_text_visible_raster_glyph_count,
        &["render", "ui", "text", "raster"],
    );
    record_count(
        store,
        "render.ui.text.raster.source_image_count",
        frame_index,
        stats.last_ui_text_raster_source_image_count,
        &["render", "ui", "text", "raster"],
    );
    record_count(
        store,
        "render.ui.text.raster.worker_pending_count",
        frame_index,
        stats.last_ui_text_raster_worker_pending_count,
        &["render", "ui", "text", "raster", "worker"],
    );
    record_count(
        store,
        "render.ui.text.raster.worker_failed_count",
        frame_index,
        stats.last_ui_text_raster_worker_failed_count,
        &["render", "ui", "text", "raster", "worker"],
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
