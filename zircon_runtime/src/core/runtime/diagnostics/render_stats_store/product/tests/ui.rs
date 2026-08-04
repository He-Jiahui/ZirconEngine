use crate::core::diagnostics::DiagnosticStore;
use crate::core::framework::render::RenderStats;

use super::super::record;
use super::assert_series;

#[test]
fn render_product_diagnostics_record_ui_text_raster_stats() {
    let mut store = DiagnosticStore::default();
    let stats = RenderStats {
        submitted_frames: 12,
        last_ui_text_glyph_count: 13,
        last_ui_text_unmapped_glyph_count: 2,
        last_ui_text_visible_raster_glyph_count: 11,
        last_ui_text_raster_source_image_count: 10,
        last_ui_text_missing_raster_image_count: 2,
        last_ui_text_visible_missing_raster_image_count: 1,
        last_ui_text_visible_raster_placeholder_count: 3,
        last_ui_text_raster_worker_pending_count: 3,
        last_ui_text_raster_worker_failed_count: 1,
        last_ui_text_raster_renderer_upload_requeued_count: 4,
        last_ui_text_raster_renderer_upload_failure_count: 5,
        ..RenderStats::default()
    };

    record(&mut store, &stats);

    assert_series(&store, "render.ui.text.glyph_count", 13.0, "count");
    assert_series(&store, "render.ui.text.unmapped_glyph_count", 2.0, "count");
    assert_series(
        &store,
        "render.ui.text.raster.visible_glyph_count",
        11.0,
        "count",
    );
    assert_series(
        &store,
        "render.ui.text.raster.source_image_count",
        10.0,
        "count",
    );
    assert_series(
        &store,
        "render.ui.text.raster.missing_image_count",
        2.0,
        "count",
    );
    assert_series(
        &store,
        "render.ui.text.raster.visible_missing_image_count",
        1.0,
        "count",
    );
    assert_series(
        &store,
        "render.ui.text.raster.visible_placeholder_count",
        3.0,
        "count",
    );
    assert_series(
        &store,
        "render.ui.text.raster.worker_pending_count",
        3.0,
        "count",
    );
    assert_series(
        &store,
        "render.ui.text.raster.worker_failed_count",
        1.0,
        "count",
    );
    assert_series(
        &store,
        "render.ui.text.raster.renderer_upload_requeued_count",
        4.0,
        "count",
    );
    assert_series(
        &store,
        "render.ui.text.raster.renderer_upload_failure_count",
        5.0,
        "count",
    );
}
