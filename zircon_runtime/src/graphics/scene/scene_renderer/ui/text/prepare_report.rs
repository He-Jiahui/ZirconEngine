use super::ScreenSpaceUiNativePrepareReport;
use super::font_assets::UiFontAssetCacheReport;
use super::font_id_report::ScreenSpaceUiTextFontIdReport;
use super::resolved_batches::{AutoTextRasterRouteFrameReport, ResolvedScreenSpaceUiTextBatches};
use super::sdf_fallback::ScreenSpaceUiTextSdfFallbackReport;
use crate::graphics::scene::scene_renderer::ui::atlas_renderer::GlyphAtlasBitmapRendererPrepareReport;
use crate::graphics::scene::scene_renderer::ui::render::{
    ScreenSpaceUiResolvedGlyphArtifactRouteReport, ScreenSpaceUiTextBatch,
};
use crate::graphics::scene::scene_renderer::ui::sdf_atlas::SdfAtlasCacheReport;
use crate::graphics::scene::scene_renderer::ui::sdf_render::ScreenSpaceUiSdfPrepareReport;
use crate::text::TextLayoutFallbackReport;
use crate::text::font::MissingGlyphDiagnosticsReport;
use crate::text::native_bitmap_atlas::{
    NativeBitmapAtlasHandoff, NativeBitmapAtlasPrepareReport,
    native_bitmap_atlas_handoff_for_report,
};

#[cfg(feature = "profiling")]
mod profile;
#[cfg(feature = "profiling")]
pub(super) use profile::record_text_prepare_profile;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ScreenSpaceUiTextPrepareReport {
    pub(super) input_auto_text_batch_count: usize,
    pub(super) input_native_text_batch_count: usize,
    pub(super) input_sdf_text_batch_count: usize,
    pub(crate) resolved_glyph_artifact_routes: ScreenSpaceUiResolvedGlyphArtifactRouteReport,
    pub(super) resolved_native_text_batch_count: usize,
    pub(super) resolved_sdf_text_batch_count: usize,
    pub(crate) renderer_batch_residency: ScreenSpaceUiTextBatchResidencyReport,
    pub(crate) post_layout_stale_artifact_batch_rejection_count: usize,
    pub(super) auto_route: AutoTextRasterRouteFrameReport,
    pub(super) sdf_fallback: ScreenSpaceUiTextSdfFallbackReport,
    pub(crate) font_assets: UiFontAssetCacheReport,
    pub(crate) native_font_ids: ScreenSpaceUiTextFontIdReport,
    pub(super) missing_glyphs: MissingGlyphDiagnosticsReport,
    pub(crate) layout_fallbacks: TextLayoutFallbackReport,
    pub(crate) raster_upload: ScreenSpaceUiTextRasterUploadReport,
    pub(super) native_bitmap_atlas: NativeBitmapAtlasPrepareReport,
    pub(super) bitmap_atlas_renderer: GlyphAtlasBitmapRendererPrepareReport,
    pub(super) sdf_atlas: SdfAtlasCacheReport,
    pub(crate) sdf_generation: ScreenSpaceUiTextSdfGenerationReport,
    pub(super) sdf_renderer: ScreenSpaceUiSdfPrepareReport,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ScreenSpaceUiTextBatchResidencyReport {
    pub(crate) materialized_batch_count: usize,
    pub(crate) text_byte_count: usize,
    pub(crate) glyph_advance_byte_count: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ScreenSpaceUiResolvedTextReport {
    native_text_batch_count: usize,
    sdf_text_batch_count: usize,
    batch_residency: ScreenSpaceUiTextBatchResidencyReport,
    post_layout_stale_artifact_batch_rejection_count: usize,
    layout_fallbacks: TextLayoutFallbackReport,
}

impl ScreenSpaceUiResolvedTextReport {
    pub(super) fn from_resolved_texts(texts: &ResolvedScreenSpaceUiTextBatches) -> Self {
        Self {
            native_text_batch_count: texts.native_texts().len(),
            sdf_text_batch_count: texts.sdf_texts().len(),
            batch_residency: text_batch_residency_report(texts.native_texts(), texts.sdf_texts()),
            post_layout_stale_artifact_batch_rejection_count: texts
                .post_layout_stale_artifact_batch_rejection_count(),
            layout_fallbacks: texts.layout_fallback_report(),
        }
    }

    pub(super) fn merge(&mut self, segment: Self) {
        self.native_text_batch_count = self
            .native_text_batch_count
            .saturating_add(segment.native_text_batch_count);
        self.sdf_text_batch_count = self
            .sdf_text_batch_count
            .saturating_add(segment.sdf_text_batch_count);
        self.batch_residency.merge(segment.batch_residency);
        self.post_layout_stale_artifact_batch_rejection_count = self
            .post_layout_stale_artifact_batch_rejection_count
            .saturating_add(segment.post_layout_stale_artifact_batch_rejection_count);
        merge_layout_fallback_report(&mut self.layout_fallbacks, segment.layout_fallbacks);
    }
}

impl ScreenSpaceUiTextBatchResidencyReport {
    fn merge(&mut self, segment: Self) {
        self.materialized_batch_count = self
            .materialized_batch_count
            .saturating_add(segment.materialized_batch_count);
        self.text_byte_count = self.text_byte_count.saturating_add(segment.text_byte_count);
        self.glyph_advance_byte_count = self
            .glyph_advance_byte_count
            .saturating_add(segment.glyph_advance_byte_count);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ScreenSpaceUiTextSdfGenerationReport {
    pub(crate) pending_batch_count: usize,
    pub(crate) completion_backlog_count: usize,
    pub(crate) failure_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ScreenSpaceUiTextRasterUploadReport {
    pub(crate) visible_raster_glyph_count: usize,
    pub(crate) source_image_count: usize,
    pub(crate) missing_raster_image_count: usize,
    /// Missing source rasters whose glyphs affect the current frame.
    ///
    /// Product capture gates use this instead of the total diagnostic count,
    /// which also retains offscreen cache misses for observability.
    pub(crate) visible_missing_raster_image_count: usize,
    /// Visible native glyphs deliberately emitted as transparent placeholders.
    ///
    /// A source raster can be present while atlas allocation is deferred, so this
    /// must remain distinct from missing source images for product capture gates.
    pub(crate) visible_placeholder_count: usize,
    pub(super) approximate_raster_image_count: usize,
    pub(super) source_cache_hit_count: usize,
    pub(super) source_cache_approximate_hit_count: usize,
    /// Published to crate-level frame statistics for product diagnostics.
    pub(crate) source_cache_miss_count: usize,
    pub(super) source_cache_insert_count: usize,
    pub(super) source_cache_capacity: usize,
    pub(super) source_cache_entry_count: usize,
    /// Actual de-duplicated physical raster identities currently retained by the source cache.
    pub(crate) source_cache_persistent_raster_key_count: usize,
    pub(super) source_cache_resident_byte_count: usize,
    pub(super) source_cache_max_byte_count: usize,
    pub(super) source_cache_approximate_probe_count: usize,
    pub(super) source_cache_lru_repair_count: usize,
    pub(super) source_cache_lru_touch_count: usize,
    pub(super) source_cache_evicted_count: usize,
    pub(super) source_cache_evicted_byte_count: usize,
    pub(super) source_cache_budget_linked_eviction_count: usize,
    pub(super) source_cache_linked_raster_invalidation_count: usize,
    pub(super) source_cache_rejected_byte_budget_count: usize,
    pub(super) source_cache_invalidated_count: usize,
    pub(super) atlas_slot_cache_hit_count: usize,
    pub(super) atlas_slot_cache_miss_count: usize,
    pub(super) atlas_slot_cache_insert_count: usize,
    pub(super) atlas_resident_page_byte_len: usize,
    pub(super) atlas_page_shadow_resident_page_count: usize,
    pub(super) atlas_page_shadow_resident_byte_count: usize,
    pub(super) atlas_page_shadow_max_byte_count: usize,
    pub(super) atlas_page_shadow_budget_rejection_count: u64,
    pub(super) worker_request_submitted_count: usize,
    /// Outstanding native raster work after the frame completion drain.
    ///
    /// This is deliberately the source cache's durable in-flight count, not
    /// only requests whose glyph happened to be encountered again this frame.
    /// A byte-budget-deferred completion keeps its source-cache work id, so it
    /// is represented here once rather than added again from its diagnostic.
    pub(crate) worker_pending_count: usize,
    pub(crate) worker_request_deferred_count: usize,
    pub(crate) worker_request_unavailable_count: usize,
    pub(crate) worker_request_backpressured_count: usize,
    pub(crate) worker_request_font_copied_byte_count: usize,
    pub(crate) worker_raster_font_resident_byte_count: usize,
    pub(crate) worker_raster_font_entry_count: usize,
    pub(crate) worker_request_cancelled_count: usize,
    pub(crate) worker_completion_applied_byte_count: usize,
    pub(super) worker_completion_drained_byte_count: usize,
    pub(super) worker_completion_byte_budget_deferred_count: usize,
    pub(super) worker_completion_oversized_accepted_count: usize,
    pub(crate) worker_pool_budgeted_thread_count: usize,
    pub(crate) worker_pool_in_flight_count: usize,
    pub(crate) worker_pool_queued_count: usize,
    pub(crate) worker_pool_queued_input_byte_count: usize,
    pub(crate) worker_pool_running_count: usize,
    pub(crate) worker_pool_completed_total: u64,
    pub(crate) worker_pool_failed_total: u64,
    pub(crate) worker_pool_queue_peak_count: usize,
    pub(crate) worker_pool_completion_backlog_count: usize,
    pub(crate) worker_pool_completion_backlog_byte_count: usize,
    pub(crate) worker_pool_completion_backpressured_total: u64,
    pub(crate) worker_pool_completion_budget_rejected_total: u64,
    pub(crate) worker_pool_completion_rejected_byte_total: u64,
    pub(crate) worker_pool_request_backpressured_total: u64,
    pub(crate) worker_pool_cancelled_total: u64,
    /// Native raster failures or rejected completion images.
    pub(crate) worker_failed_count: usize,
    pub(super) upload_command_count: usize,
    pub(super) upload_copy_count: usize,
    pub(super) upload_copy_byte_len: usize,
    pub(super) upload_byte_len: usize,
    pub(super) renderer_upload_request_count: usize,
    pub(super) renderer_upload_byte_len: usize,
    pub(crate) renderer_upload_requeued_count: usize,
    pub(crate) renderer_upload_failure_count: usize,
    pub(super) renderer_upload_ready_to_write_texture: bool,
}

pub(super) fn text_prepare_report(
    input_batch_counts: [usize; 3],
    auto_route: AutoTextRasterRouteFrameReport,
    resolved_glyph_artifact_routes: ScreenSpaceUiResolvedGlyphArtifactRouteReport,
    resolved_texts: ScreenSpaceUiResolvedTextReport,
    sdf_fallback: ScreenSpaceUiTextSdfFallbackReport,
    native_prepare: ScreenSpaceUiNativePrepareReport,
    font_assets: UiFontAssetCacheReport,
    missing_glyphs: MissingGlyphDiagnosticsReport,
    bitmap_atlas_renderer: GlyphAtlasBitmapRendererPrepareReport,
    sdf_atlas: SdfAtlasCacheReport,
    sdf_renderer: ScreenSpaceUiSdfPrepareReport,
) -> ScreenSpaceUiTextPrepareReport {
    let raster_upload =
        text_raster_upload_report(&native_prepare.bitmap_atlas, &bitmap_atlas_renderer);
    let sdf_generation = ScreenSpaceUiTextSdfGenerationReport {
        pending_batch_count: sdf_renderer.bake.generation_scheduler.in_flight_batch_count,
        completion_backlog_count: sdf_renderer
            .bake
            .generation_scheduler
            .completion_backlog_count,
        failure_count: sdf_renderer.bake.generation_failure_count,
    };
    ScreenSpaceUiTextPrepareReport {
        input_auto_text_batch_count: input_batch_counts[0],
        input_native_text_batch_count: input_batch_counts[1],
        input_sdf_text_batch_count: input_batch_counts[2],
        resolved_glyph_artifact_routes,
        resolved_native_text_batch_count: resolved_texts.native_text_batch_count,
        resolved_sdf_text_batch_count: resolved_texts.sdf_text_batch_count,
        renderer_batch_residency: resolved_texts.batch_residency,
        post_layout_stale_artifact_batch_rejection_count: resolved_texts
            .post_layout_stale_artifact_batch_rejection_count,
        auto_route,
        sdf_fallback,
        font_assets,
        native_font_ids: native_prepare.font_ids,
        missing_glyphs,
        layout_fallbacks: resolved_texts.layout_fallbacks,
        raster_upload,
        native_bitmap_atlas: native_prepare.bitmap_atlas,
        bitmap_atlas_renderer,
        sdf_atlas,
        sdf_generation,
        sdf_renderer,
    }
}

fn text_batch_residency_report(
    native_texts: &[ScreenSpaceUiTextBatch],
    sdf_texts: &[ScreenSpaceUiTextBatch],
) -> ScreenSpaceUiTextBatchResidencyReport {
    let mut report = ScreenSpaceUiTextBatchResidencyReport::default();
    for batch in native_texts.iter().chain(sdf_texts) {
        report.materialized_batch_count = report.materialized_batch_count.saturating_add(1);
        report.text_byte_count = report.text_byte_count.saturating_add(batch.text.len());
        report.glyph_advance_byte_count = report.glyph_advance_byte_count.saturating_add(
            batch
                .glyph_advances
                .len()
                .saturating_mul(std::mem::size_of::<f32>()),
        );
    }
    report
}

fn merge_layout_fallback_report(
    frame: &mut TextLayoutFallbackReport,
    segment: TextLayoutFallbackReport,
) {
    debug_assert_eq!(
        frame.unicode_data_generation,
        segment.unicode_data_generation
    );
    debug_assert_eq!(
        frame.unicode_data_fingerprint,
        segment.unicode_data_fingerprint
    );
    frame.fallback_count = frame.fallback_count.saturating_add(segment.fallback_count);
    frame.generation_deferred_count = frame
        .generation_deferred_count
        .saturating_add(segment.generation_deferred_count);
    frame.invalid_font_size_count = frame
        .invalid_font_size_count
        .saturating_add(segment.invalid_font_size_count);
    frame.invalid_language_count = frame
        .invalid_language_count
        .saturating_add(segment.invalid_language_count);
    frame.bidi_invariant_count = frame
        .bidi_invariant_count
        .saturating_add(segment.bidi_invariant_count);
    frame.other_error_count = frame
        .other_error_count
        .saturating_add(segment.other_error_count);
}

pub(super) fn text_raster_upload_report(
    native_bitmap_atlas: &NativeBitmapAtlasPrepareReport,
    bitmap_atlas_renderer: &GlyphAtlasBitmapRendererPrepareReport,
) -> ScreenSpaceUiTextRasterUploadReport {
    let visible_placeholder_count = matches!(
        native_bitmap_atlas_handoff_for_report(native_bitmap_atlas),
        NativeBitmapAtlasHandoff::TransparentPlaceholder
    )
    .then_some(native_bitmap_atlas.submission.visible_placeholder_count)
    .unwrap_or_default();

    ScreenSpaceUiTextRasterUploadReport {
        visible_raster_glyph_count: native_bitmap_atlas.visible_raster_glyph_count,
        source_image_count: native_bitmap_atlas.source_image_count,
        missing_raster_image_count: native_bitmap_atlas.missing_raster_image_count,
        visible_missing_raster_image_count: native_bitmap_atlas.visible_missing_raster_image_count,
        visible_placeholder_count,
        approximate_raster_image_count: native_bitmap_atlas.approximate_raster_image_count,
        source_cache_hit_count: native_bitmap_atlas.source_cache.hit_count,
        source_cache_approximate_hit_count: native_bitmap_atlas.source_cache.approximate_hit_count,
        source_cache_miss_count: native_bitmap_atlas.source_cache.miss_count,
        source_cache_insert_count: native_bitmap_atlas.source_cache.insert_count,
        source_cache_capacity: native_bitmap_atlas.source_cache.capacity,
        source_cache_entry_count: native_bitmap_atlas.source_cache.entry_count,
        source_cache_persistent_raster_key_count: native_bitmap_atlas
            .source_cache
            .persistent_raster_key_count,
        source_cache_resident_byte_count: native_bitmap_atlas.source_cache.resident_byte_count,
        source_cache_max_byte_count: native_bitmap_atlas.source_cache.max_byte_count,
        source_cache_approximate_probe_count: native_bitmap_atlas
            .source_cache
            .approximate_probe_count,
        source_cache_lru_repair_count: native_bitmap_atlas.source_cache.lru_repair_count,
        source_cache_lru_touch_count: native_bitmap_atlas.source_cache.lru_touch_count,
        source_cache_evicted_count: native_bitmap_atlas.source_cache.evicted_count,
        source_cache_evicted_byte_count: native_bitmap_atlas.source_cache.evicted_byte_count,
        source_cache_budget_linked_eviction_count: native_bitmap_atlas
            .source_cache
            .budget_linked_eviction_count,
        source_cache_linked_raster_invalidation_count: native_bitmap_atlas
            .source_cache
            .linked_raster_invalidation_count,
        source_cache_rejected_byte_budget_count: native_bitmap_atlas
            .source_cache
            .rejected_byte_budget_count,
        source_cache_invalidated_count: native_bitmap_atlas.source_cache.invalidated_count,
        atlas_slot_cache_hit_count: native_bitmap_atlas.submission.slot_cache_hit_count,
        atlas_slot_cache_miss_count: native_bitmap_atlas.submission.slot_cache_miss_count,
        atlas_slot_cache_insert_count: native_bitmap_atlas.submission.slot_cache_insert_count,
        atlas_resident_page_byte_len: native_bitmap_atlas.submission.resident_page_byte_len,
        atlas_page_shadow_resident_page_count: native_bitmap_atlas
            .submission
            .bitmap_page_shadow
            .resident_page_count,
        atlas_page_shadow_resident_byte_count: native_bitmap_atlas
            .submission
            .bitmap_page_shadow
            .resident_byte_count,
        atlas_page_shadow_max_byte_count: native_bitmap_atlas
            .submission
            .bitmap_page_shadow
            .max_byte_count,
        atlas_page_shadow_budget_rejection_count: native_bitmap_atlas
            .submission
            .bitmap_page_shadow
            .budget_rejection_count,
        worker_request_submitted_count: native_bitmap_atlas
            .source_cache
            .worker_request_submitted_count,
        worker_pending_count: native_bitmap_atlas.source_cache.pending_worker_count,
        worker_request_deferred_count: native_bitmap_atlas
            .source_cache
            .worker_request_deferred_count,
        worker_request_unavailable_count: native_bitmap_atlas
            .source_cache
            .worker_request_unavailable_count,
        worker_request_backpressured_count: native_bitmap_atlas
            .source_cache
            .worker_request_backpressured_count,
        worker_request_font_copied_byte_count: native_bitmap_atlas
            .source_cache
            .worker_request_font_copied_byte_count,
        worker_raster_font_resident_byte_count: native_bitmap_atlas
            .source_cache
            .worker_raster_font_resident_byte_count,
        worker_raster_font_entry_count: native_bitmap_atlas
            .source_cache
            .worker_raster_font_entry_count,
        worker_request_cancelled_count: native_bitmap_atlas
            .source_cache
            .worker_request_cancelled_count,
        worker_completion_applied_byte_count: native_bitmap_atlas
            .source_cache
            .worker_completion_applied_byte_count,
        worker_completion_drained_byte_count: native_bitmap_atlas
            .source_cache
            .worker_completion_drained_byte_count,
        worker_completion_byte_budget_deferred_count: native_bitmap_atlas
            .source_cache
            .worker_completion_byte_budget_deferred_count,
        worker_completion_oversized_accepted_count: native_bitmap_atlas
            .source_cache
            .worker_completion_oversized_accepted_count,
        worker_pool_budgeted_thread_count: native_bitmap_atlas
            .source_cache
            .worker_pool_budgeted_thread_count,
        worker_pool_in_flight_count: native_bitmap_atlas.source_cache.worker_pool_in_flight_count,
        worker_pool_queued_count: native_bitmap_atlas.source_cache.worker_pool_queued_count,
        worker_pool_queued_input_byte_count: native_bitmap_atlas
            .source_cache
            .worker_pool_queued_input_byte_count,
        worker_pool_running_count: native_bitmap_atlas.source_cache.worker_pool_running_count,
        worker_pool_completed_total: native_bitmap_atlas.source_cache.worker_pool_completed_total,
        worker_pool_failed_total: native_bitmap_atlas.source_cache.worker_pool_failed_total,
        worker_pool_queue_peak_count: native_bitmap_atlas
            .source_cache
            .worker_pool_queue_peak_count,
        worker_pool_completion_backlog_count: native_bitmap_atlas
            .source_cache
            .worker_pool_completion_backlog_count,
        worker_pool_completion_backlog_byte_count: native_bitmap_atlas
            .source_cache
            .worker_pool_completion_backlog_byte_count,
        worker_pool_completion_backpressured_total: native_bitmap_atlas
            .source_cache
            .worker_pool_completion_backpressured_total,
        worker_pool_completion_budget_rejected_total: native_bitmap_atlas
            .source_cache
            .worker_pool_completion_budget_rejected_total,
        worker_pool_completion_rejected_byte_total: native_bitmap_atlas
            .source_cache
            .worker_pool_completion_rejected_byte_total,
        worker_pool_request_backpressured_total: native_bitmap_atlas
            .source_cache
            .worker_pool_request_backpressured_total,
        worker_pool_cancelled_total: native_bitmap_atlas.source_cache.worker_pool_cancelled_total,
        worker_failed_count: native_bitmap_atlas
            .source_cache
            .worker_request_failed_count
            .saturating_add(
                native_bitmap_atlas
                    .source_cache
                    .worker_completion_failed_count,
            )
            .saturating_add(
                native_bitmap_atlas
                    .source_cache
                    .worker_completion_invalid_bitmap_count,
            ),
        upload_command_count: native_bitmap_atlas.submission.upload_command_count,
        upload_copy_count: native_bitmap_atlas.submission.upload_copy_count,
        upload_copy_byte_len: native_bitmap_atlas.submission.upload_copy_byte_len,
        upload_byte_len: native_bitmap_atlas.submission.upload_byte_len,
        renderer_upload_request_count: bitmap_atlas_renderer.upload_request_count,
        renderer_upload_byte_len: bitmap_atlas_renderer.upload_byte_len,
        renderer_upload_requeued_count: bitmap_atlas_renderer.upload_requeued_count,
        renderer_upload_failure_count: bitmap_atlas_renderer.upload_failure_count,
        renderer_upload_ready_to_write_texture: bitmap_atlas_renderer.upload_ready_to_write_texture,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolved_text_report_merges_segment_counts_and_payload_sizes() {
        let mut frame = ScreenSpaceUiResolvedTextReport {
            native_text_batch_count: 2,
            sdf_text_batch_count: 1,
            batch_residency: ScreenSpaceUiTextBatchResidencyReport {
                materialized_batch_count: 3,
                text_byte_count: 12,
                glyph_advance_byte_count: 8,
            },
            post_layout_stale_artifact_batch_rejection_count: 1,
            layout_fallbacks: TextLayoutFallbackReport::default(),
        };
        let mut segment_fallbacks = TextLayoutFallbackReport::default();
        segment_fallbacks.fallback_count = 2;
        segment_fallbacks.invalid_language_count = 1;

        frame.merge(ScreenSpaceUiResolvedTextReport {
            native_text_batch_count: 1,
            sdf_text_batch_count: 4,
            batch_residency: ScreenSpaceUiTextBatchResidencyReport {
                materialized_batch_count: 5,
                text_byte_count: 20,
                glyph_advance_byte_count: 16,
            },
            post_layout_stale_artifact_batch_rejection_count: 3,
            layout_fallbacks: segment_fallbacks,
        });

        assert_eq!(frame.native_text_batch_count, 3);
        assert_eq!(frame.sdf_text_batch_count, 5);
        assert_eq!(frame.batch_residency.materialized_batch_count, 8);
        assert_eq!(frame.batch_residency.text_byte_count, 32);
        assert_eq!(frame.batch_residency.glyph_advance_byte_count, 24);
        assert_eq!(frame.post_layout_stale_artifact_batch_rejection_count, 4);
        assert_eq!(frame.layout_fallbacks.fallback_count, 2);
        assert_eq!(frame.layout_fallbacks.invalid_language_count, 1);
    }
}
