use super::super::ScreenSpaceUiTextPrepareReport;

pub(super) fn record_dto_residency_profile(report: &ScreenSpaceUiTextPrepareReport) {
    crate::profile_counter!(
        "runtime",
        "ui_text.prepare.input_batches",
        report
            .input_auto_text_batch_count
            .saturating_add(report.input_native_text_batch_count)
            .saturating_add(report.input_sdf_text_batch_count)
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.prepare.resolved_native_batches",
        report.resolved_native_text_batch_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.prepare.resolved_sdf_batches",
        report.resolved_sdf_text_batch_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.dto_projection.renderer_batches",
        report.renderer_batch_residency.materialized_batch_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.dto_projection.renderer_text_bytes",
        report.renderer_batch_residency.text_byte_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.dto_projection.renderer_glyph_advance_bytes",
        report.renderer_batch_residency.glyph_advance_byte_count
    );
}
