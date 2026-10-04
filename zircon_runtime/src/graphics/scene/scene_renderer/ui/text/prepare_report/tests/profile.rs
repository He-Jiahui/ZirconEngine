use super::{record_text_prepare_profile, ScreenSpaceUiTextPrepareReport};
use crate::core::runtime::diagnostics::profiling::{
    reset_capture, snapshot, start_capture, test_capture_lock, ProfileCaptureConfig,
};

#[test]
fn text_prepare_profile_projects_existing_raster_and_draw_reports() {
    let _capture_guard = test_capture_lock();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "ui-text-prepare-profile".to_string();
    config.max_counters = 128;
    start_capture(config);

    let mut report = ScreenSpaceUiTextPrepareReport {
        input_auto_text_batch_count: 2,
        input_native_text_batch_count: 3,
        input_sdf_text_batch_count: 5,
        resolved_native_text_batch_count: 4,
        resolved_sdf_text_batch_count: 6,
        post_layout_stale_artifact_batch_rejection_count: 89,
        ..ScreenSpaceUiTextPrepareReport::default()
    };
    report.raster_upload.source_cache_hit_count = 7;
    report.raster_upload.source_cache_approximate_hit_count = 79;
    report.raster_upload.source_cache_miss_count = 11;
    report.raster_upload.source_cache_insert_count = 83;
    report.raster_upload.source_cache_capacity = 13;
    report.raster_upload.source_cache_entry_count = 17;
    report
        .raster_upload
        .source_cache_persistent_raster_key_count = 71;
    report.raster_upload.source_cache_resident_byte_count = 37;
    report.raster_upload.source_cache_max_byte_count = 41;
    report.raster_upload.source_cache_approximate_probe_count = 43;
    report.raster_upload.source_cache_lru_repair_count = 47;
    report.raster_upload.source_cache_lru_touch_count = 53;
    report.raster_upload.source_cache_evicted_count = 19;
    report.raster_upload.source_cache_evicted_byte_count = 23;
    report
        .raster_upload
        .source_cache_budget_linked_eviction_count = 59;
    report
        .raster_upload
        .source_cache_linked_raster_invalidation_count = 61;
    report.raster_upload.source_cache_rejected_byte_budget_count = 29;
    report.raster_upload.source_cache_invalidated_count = 31;
    report.raster_upload.atlas_page_shadow_resident_page_count = 97;
    report.raster_upload.atlas_page_shadow_resident_byte_count = 101;
    report.raster_upload.atlas_page_shadow_max_byte_count = 103;
    report
        .raster_upload
        .atlas_page_shadow_budget_rejection_count = 107;
    report.raster_upload.worker_pending_count = 13;
    report.raster_upload.worker_raster_font_resident_byte_count = 71;
    report.raster_upload.worker_raster_font_entry_count = 73;
    report
        .raster_upload
        .worker_pool_completion_backlog_byte_count = 41;
    report
        .raster_upload
        .worker_pool_completion_budget_rejected_total = 43;
    report.raster_upload.worker_completion_drained_byte_count = 47;
    report
        .raster_upload
        .worker_completion_byte_budget_deferred_count = 53;
    report
        .raster_upload
        .worker_completion_oversized_accepted_count = 57;
    report.raster_upload.worker_pool_completed_total = 59;
    report.raster_upload.worker_pool_failed_total = 61;
    report.raster_upload.worker_pool_queue_peak_count = 67;
    report.raster_upload.upload_copy_count = 17;
    report.raster_upload.upload_byte_len = 19;
    report.resolved_glyph_artifact_routes.rejected_command_count = 73;
    report.renderer_batch_residency.materialized_batch_count = 79;
    report.renderer_batch_residency.text_byte_count = 83;
    report.renderer_batch_residency.glyph_advance_byte_count = 87;
    report
        .bitmap_atlas_renderer
        .storage_pass_visible_glyph_count = 31;
    report.bitmap_atlas_renderer.draw_command_count = 37;
    report.sdf_renderer.bake.resident_font_asset_error_count = 5;
    report
        .sdf_renderer
        .bake
        .resident_font_asset_no_registered_faces_count = 31;
    report.sdf_renderer.bake.generation_scheduler.budget =
        crate::text::sdf::SdfGenerationBudgetSnapshot {
            max_in_flight_batches: 109,
            max_glyphs_per_batch: 113,
            max_in_flight_glyphs: 127,
            source_byte_budget: 131,
            completion_queue_depth: 137,
            completion_byte_budget: 139,
        };
    report.sdf_renderer.vertex_count = 23;
    report.sdf_renderer.draw_count = 29;
    report.sdf_renderer.compiled_segment_visit_count = 41;
    report.sdf_renderer.compiled_vertex_visit_count = 43;
    report.sdf_renderer.compiled_material_visit_count = 47;
    report.sdf_renderer.compiled_full_rebuild_count = 53;
    report.sdf_renderer.vertex_buffer_write_count = 59;
    report.sdf_renderer.vertex_buffer_write_byte_len = 61;
    report.sdf_renderer.material_buffer_write_count = 67;
    report.sdf_renderer.material_buffer_write_byte_len = 71;

    record_text_prepare_profile(&report);
    let profile = snapshot();
    reset_capture();

    assert_eq!(
        counter_value(&profile, "ui_text.prepare.input_batches"),
        10.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.native_raster_plan.worker_completion_backlog_bytes"
        ),
        41.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.native_raster_plan.worker_completion_budget_rejected_total"
        ),
        43.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.native_raster_plan.worker_completion_drained_bytes"
        ),
        47.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.native_raster_plan.worker_completion_budget_deferred"
        ),
        53.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.native_raster_plan.worker_completion_oversized_accepted"
        ),
        57.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.native_raster_plan.worker_pool_completed_total"
        ),
        59.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.native_raster_plan.worker_pool_failed_total"
        ),
        61.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.native_raster_plan.worker_pool_queue_peak"
        ),
        67.0
    );
    assert_eq!(
        counter_value(&profile, "ui_text.native_raster_plan.source_cache_hits"),
        7.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.native_raster_plan.source_cache_approximate_hits"
        ),
        79.0
    );
    assert_eq!(
        counter_value(&profile, "ui_text.native_raster_plan.source_cache_misses"),
        11.0
    );
    assert_eq!(
        counter_value(&profile, "ui_text.native_raster_plan.source_cache_inserts"),
        83.0
    );
    assert_eq!(
        counter_value(&profile, "ui_text.atlas_page_shadow.resident_bytes"),
        101.0
    );
    assert_eq!(
        counter_value(&profile, "text.runtime_budget.atlas_page_shadow_bytes"),
        103.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.atlas_page_shadow.budget_rejections_total"
        ),
        107.0
    );
    assert_eq!(
        counter_value(&profile, "text.runtime_budget.sdf_completion_bytes"),
        139.0
    );
    assert_eq!(
        counter_value(&profile, "ui_text.native_raster_plan.source_cache_capacity"),
        13.0
    );
    assert_eq!(
        counter_value(&profile, "ui_text.native_raster_plan.source_cache_entries"),
        17.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.native_raster_plan.source_cache_persistent_raster_keys"
        ),
        71.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.native_raster_plan.source_cache_resident_bytes"
        ),
        37.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.native_raster_plan.source_cache_max_bytes"
        ),
        41.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.native_raster_plan.source_cache_approximate_probes"
        ),
        43.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.native_raster_plan.source_cache_lru_repairs"
        ),
        47.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.native_raster_plan.source_cache_lru_touches"
        ),
        53.0
    );
    assert_eq!(
        counter_value(&profile, "ui_text.native_raster_plan.source_cache_evicted"),
        19.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.native_raster_plan.source_cache_evicted_bytes"
        ),
        23.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.native_raster_plan.source_cache_budget_linked_evictions"
        ),
        59.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.native_raster_plan.source_cache_linked_invalidations"
        ),
        61.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.native_raster_plan.source_cache_budget_rejections"
        ),
        29.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.native_raster_plan.source_cache_invalidated"
        ),
        31.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.native_raster_plan.worker_font_resident_bytes"
        ),
        71.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.native_raster_plan.worker_font_resident_entries"
        ),
        73.0
    );
    assert_eq!(
        counter_value(&profile, "ui_text.atlas_upload.native_copy_count"),
        17.0
    );
    assert_eq!(
        counter_value(&profile, "ui_text.atlas_upload.native_bytes"),
        19.0
    );
    assert_eq!(
        counter_value(&profile, "ui_text.atlas_upload.native_instances"),
        31.0
    );
    assert_eq!(
        counter_value(&profile, "ui_text.atlas_upload.native_draws"),
        37.0
    );
    assert_eq!(
        counter_value(&profile, "ui_text.sdf_prepare.vertices"),
        23.0
    );
    assert_eq!(counter_value(&profile, "ui_text.sdf_prepare.draws"), 29.0);
    assert_eq!(
        counter_value(&profile, "ui_text.sdf_prepare.compiled_segment_visits"),
        41.0
    );
    assert_eq!(
        counter_value(&profile, "ui_text.sdf_prepare.compiled_vertex_visits"),
        43.0
    );
    assert_eq!(
        counter_value(&profile, "ui_text.sdf_prepare.compiled_material_visits"),
        47.0
    );
    assert_eq!(
        counter_value(&profile, "ui_text.sdf_prepare.compiled_full_rebuilds"),
        53.0
    );
    assert_eq!(
        counter_value(&profile, "ui_text.sdf_prepare.vertex_buffer_write_count"),
        59.0
    );
    assert_eq!(
        counter_value(&profile, "ui_text.sdf_prepare.vertex_buffer_write_bytes"),
        61.0
    );
    assert_eq!(
        counter_value(&profile, "ui_text.sdf_prepare.material_buffer_write_count"),
        67.0
    );
    assert_eq!(
        counter_value(&profile, "ui_text.sdf_prepare.material_buffer_write_bytes"),
        71.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.prepare.post_layout_stale_artifact_batch_rejections"
        ),
        89.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.resolved_glyph_artifact_route.rejected_commands"
        ),
        73.0
    );
    assert_eq!(
        counter_value(&profile, "ui_text.dto_projection.renderer_batches"),
        79.0
    );
    assert_eq!(
        counter_value(&profile, "ui_text.dto_projection.renderer_text_bytes"),
        83.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.dto_projection.renderer_glyph_advance_bytes"
        ),
        87.0
    );
    assert_eq!(
        counter_value(&profile, "ui_text.sdf_prepare.resident_font_asset_errors"),
        5.0
    );
    assert_eq!(
        counter_value(
            &profile,
            "ui_text.sdf_prepare.resident_font_asset_no_registered_faces"
        ),
        31.0
    );
}

fn counter_value(
    profile: &crate::core::runtime::diagnostics::profiling::ProfileSnapshot,
    name: &str,
) -> f64 {
    profile
        .counters
        .iter()
        .find(|counter| counter.stream == "runtime" && counter.name == name)
        .map(|counter| counter.value)
        .unwrap_or_else(|| panic!("missing profile counter: {name}"))
}
