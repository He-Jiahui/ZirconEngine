use super::super::{GlyphAtlasBitmapAllocationFailureReason, GlyphAtlasUploadMode};
use super::plan::GlyphAtlasBitmapRenderSubmissionPlan;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapRenderSubmissionReport {
    pub(crate) source_count: usize,
    pub(crate) allocated_glyph_count: usize,
    pub(crate) visible_glyph_count: usize,
    pub(crate) skipped_glyph_count: usize,
    pub(crate) allocation_failure_count: usize,
    pub(crate) blocked_retry_count: usize,
    pub(crate) placeholder_glyph_count: usize,
    pub(crate) visible_placeholder_count: usize,
    pub(crate) skipped_placeholder_count: usize,
    pub(crate) next_retry_frame_index: Option<u64>,
    pub(crate) unsupported_format_failure_count: usize,
    pub(crate) empty_content_failure_count: usize,
    pub(crate) data_length_mismatch_failure_count: usize,
    pub(crate) page_reservation_blocked_failure_count: usize,
    pub(crate) oversized_glyph_failure_count: usize,
    pub(crate) dirty_page_count: usize,
    pub(crate) rebuilt_page_count: usize,
    pub(crate) slot_invalidation_count: usize,
    pub(crate) upload_command_count: usize,
    pub(crate) full_page_upload_count: usize,
    pub(crate) partial_upload_count: usize,
    pub(crate) upload_byte_len: usize,
    pub(crate) upload_copy_count: usize,
    pub(crate) upload_copy_byte_len: usize,
    pub(crate) draw_batch_count: usize,
    pub(crate) pipeline_count: usize,
    pub(crate) gpu_batch_count: usize,
    pub(crate) draw_command_count: usize,
    pub(crate) vertex_count: usize,
    pub(crate) requires_background_composite: bool,
}

impl GlyphAtlasBitmapRenderSubmissionReport {
    pub(crate) fn has_upload_work(self) -> bool {
        self.upload_command_count > 0 && self.upload_byte_len > 0
    }

    pub(crate) fn has_upload_copy_work(self) -> bool {
        self.upload_copy_count > 0 && self.upload_copy_byte_len > 0
    }

    pub(crate) fn has_gpu_work(self) -> bool {
        self.draw_command_count > 0 && self.vertex_count > 0
    }

    pub(crate) fn has_failures(self) -> bool {
        self.allocation_failure_count > 0
    }

    pub(crate) fn has_placeholder_work(self) -> bool {
        self.visible_placeholder_count > 0
    }

    pub(crate) fn source_validation_failure_count(self) -> usize {
        self.unsupported_format_failure_count
            + self.empty_content_failure_count
            + self.data_length_mismatch_failure_count
    }

    pub(crate) fn atlas_capacity_failure_count(self) -> usize {
        self.page_reservation_blocked_failure_count + self.oversized_glyph_failure_count
    }
}

pub(crate) fn glyph_atlas_bitmap_render_submission_report(
    plan: &GlyphAtlasBitmapRenderSubmissionPlan,
) -> GlyphAtlasBitmapRenderSubmissionReport {
    let mut report = GlyphAtlasBitmapRenderSubmissionReport {
        source_count: plan.run.glyphs.len() + plan.run.allocation_failures.len(),
        allocated_glyph_count: plan.run.glyphs.len(),
        visible_glyph_count: plan.draw_batches.visible_glyph_count,
        skipped_glyph_count: plan.draw_batches.skipped_glyph_count,
        allocation_failure_count: plan.run.allocation_failures.len(),
        blocked_retry_count: plan.run.blocked_glyphs.len(),
        placeholder_glyph_count: plan.run.placeholder_glyphs.len(),
        visible_placeholder_count: plan.placeholder_draws.visible_placeholder_count,
        skipped_placeholder_count: plan.placeholder_draws.skipped_placeholder_count,
        next_retry_frame_index: plan
            .run
            .blocked_glyphs
            .iter()
            .map(|glyph| glyph.retry_frame_index)
            .min(),
        dirty_page_count: plan.run.dirty_pages.len(),
        rebuilt_page_count: plan.run.rebuilt_pages.len(),
        slot_invalidation_count: plan.run.slot_invalidations.len(),
        upload_command_count: plan.run.upload_commands.len(),
        upload_copy_count: plan.run.upload_copies.len(),
        upload_copy_byte_len: plan
            .run
            .upload_copies
            .iter()
            .map(|copy| copy.source_byte_len)
            .sum(),
        draw_batch_count: plan.draw_batches.batches.len(),
        pipeline_count: plan.gpu_draw.pipeline_contracts.len(),
        gpu_batch_count: plan.gpu_draw.batches.len(),
        draw_command_count: plan.gpu_draw.draw_commands.len(),
        vertex_count: plan.gpu_draw.vertex_count(),
        requires_background_composite: plan.gpu_draw.requires_background_composite,
        ..GlyphAtlasBitmapRenderSubmissionReport::default()
    };

    for command in plan.upload_commands() {
        report.upload_byte_len += command.upload_byte_len;
        match command.mode {
            GlyphAtlasUploadMode::None => {}
            GlyphAtlasUploadMode::FullPage => report.full_page_upload_count += 1,
            GlyphAtlasUploadMode::PartialRect => report.partial_upload_count += 1,
        }
    }

    for failure in &plan.run.allocation_failures {
        match failure.reason {
            GlyphAtlasBitmapAllocationFailureReason::UnsupportedFormat => {
                report.unsupported_format_failure_count += 1;
            }
            GlyphAtlasBitmapAllocationFailureReason::EmptyContent => {
                report.empty_content_failure_count += 1;
            }
            GlyphAtlasBitmapAllocationFailureReason::DataLengthMismatch { .. } => {
                report.data_length_mismatch_failure_count += 1;
            }
            GlyphAtlasBitmapAllocationFailureReason::PageReservationBlocked => {
                report.page_reservation_blocked_failure_count += 1;
            }
            GlyphAtlasBitmapAllocationFailureReason::OversizedGlyph => {
                report.oversized_glyph_failure_count += 1;
            }
        }
    }

    report
}
