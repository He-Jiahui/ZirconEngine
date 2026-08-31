use super::super::ScreenSpaceUiTextPrepareReport;

pub(super) fn record_runtime_budget_profile(report: &ScreenSpaceUiTextPrepareReport) {
    let raster = &report.raster_upload;
    crate::profile_counter!(
        "runtime",
        "ui_text.atlas_page_shadow.resident_pages",
        raster.atlas_page_shadow_resident_page_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.atlas_page_shadow.resident_bytes",
        raster.atlas_page_shadow_resident_byte_count
    );
    crate::profile_counter!(
        "runtime",
        "text.runtime_budget.atlas_page_shadow_bytes",
        raster.atlas_page_shadow_max_byte_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.atlas_page_shadow.budget_rejections_total",
        raster.atlas_page_shadow_budget_rejection_count
    );

    let sdf = report.sdf_renderer.bake.generation_scheduler.budget;
    crate::profile_counter!(
        "runtime",
        "text.runtime_budget.sdf_max_in_flight_batches",
        sdf.max_in_flight_batches
    );
    crate::profile_counter!(
        "runtime",
        "text.runtime_budget.sdf_max_glyphs_per_batch",
        sdf.max_glyphs_per_batch
    );
    crate::profile_counter!(
        "runtime",
        "text.runtime_budget.sdf_max_in_flight_glyphs",
        sdf.max_in_flight_glyphs
    );
    crate::profile_counter!(
        "runtime",
        "text.runtime_budget.sdf_source_bytes",
        sdf.source_byte_budget
    );
    crate::profile_counter!(
        "runtime",
        "text.runtime_budget.sdf_completion_queue_depth",
        sdf.completion_queue_depth
    );
    crate::profile_counter!(
        "runtime",
        "text.runtime_budget.sdf_completion_bytes",
        sdf.completion_byte_budget
    );
}
