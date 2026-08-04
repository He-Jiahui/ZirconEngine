use super::state::GlyphAtlasBitmapRendererPrepareReport;

pub(super) fn glyph_atlas_bitmap_renderer_prepare_report_for_storage_passes(
    reports: &[GlyphAtlasBitmapRendererPrepareReport],
    pipeline_count: usize,
) -> GlyphAtlasBitmapRendererPrepareReport {
    let Some(first) = reports.first() else {
        return GlyphAtlasBitmapRendererPrepareReport::default();
    };
    let upload_request_count = reports
        .iter()
        .map(|report| report.upload_request_count)
        .sum();
    let upload_plan_build_count = reports
        .iter()
        .map(|report| report.upload_plan_build_count)
        .sum();
    let upload_plan_skip_count = reports
        .iter()
        .map(|report| report.upload_plan_skip_count)
        .sum();
    let upload_requeued_count = reports
        .iter()
        .map(|report| report.upload_requeued_count)
        .sum();
    let upload_missing_page_requeue_count = reports
        .iter()
        .map(|report| report.upload_missing_page_requeue_count)
        .sum();
    let upload_page_generation_mismatch_requeue_count = reports
        .iter()
        .map(|report| report.upload_page_generation_mismatch_requeue_count)
        .sum();
    let upload_face_invalidated_count = reports
        .iter()
        .map(|report| report.upload_face_invalidated_count)
        .sum();
    let upload_failure_count = reports
        .iter()
        .map(|report| report.upload_failure_count)
        .sum();
    let invalidated_storage_pass_count = reports
        .iter()
        .map(|report| report.invalidated_storage_pass_count)
        .sum();
    let upload_ready_to_write_texture = upload_request_count > 0
        && upload_failure_count == 0
        && reports
            .iter()
            .filter(|report| report.upload_request_count > 0)
            .all(|report| report.upload_ready_to_write_texture);

    GlyphAtlasBitmapRendererPrepareReport {
        atlas_size: first.atlas_size,
        atlas_layer_count: reports
            .iter()
            .map(|report| report.atlas_layer_count)
            .sum::<u32>()
            .max(1),
        atlas_storage_format: first.atlas_storage_format,
        storage_pass_count: reports.len(),
        storage_pass_visible_glyph_count: reports
            .iter()
            .map(|report| report.storage_pass_visible_glyph_count)
            .sum(),
        mixed_atlas_storage_format: reports
            .iter()
            .any(|report| report.atlas_storage_format != first.atlas_storage_format),
        atlas_resized: reports.iter().any(|report| report.atlas_resized),
        vertex_count: reports.iter().map(|report| report.vertex_count).sum(),
        vertex_buffer_byte_len: reports
            .iter()
            .map(|report| report.vertex_buffer_byte_len)
            .sum(),
        instance_buffer_capacity_byte_len: reports
            .iter()
            .map(|report| report.instance_buffer_capacity_byte_len)
            .sum(),
        instance_buffer_reallocation_count: reports
            .iter()
            .map(|report| report.instance_buffer_reallocation_count)
            .sum(),
        draw_command_count: reports.iter().map(|report| report.draw_command_count).sum(),
        pipeline_count,
        requires_background_composite: reports
            .iter()
            .any(|report| report.requires_background_composite),
        upload_plan_build_count,
        upload_plan_skip_count,
        upload_request_count,
        upload_requeued_count,
        upload_missing_page_requeue_count,
        upload_page_generation_mismatch_requeue_count,
        upload_face_invalidated_count,
        upload_byte_len: reports.iter().map(|report| report.upload_byte_len).sum(),
        upload_ready_to_write_texture,
        upload_failure_count,
        invalidated_storage_pass_count,
    }
}
