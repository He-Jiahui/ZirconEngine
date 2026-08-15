use super::font_id_report::ScreenSpaceUiTextFontIdReport;
use super::resolved_batches::{AutoTextRasterRouteFrameReport, ResolvedScreenSpaceUiTextBatches};
use super::sdf_fallback::ScreenSpaceUiTextSdfFallbackReport;
use super::ScreenSpaceUiNativePrepareReport;
use crate::graphics::scene::scene_renderer::ui::atlas_renderer::GlyphAtlasBitmapRendererPrepareReport;
use crate::graphics::scene::scene_renderer::ui::render::ScreenSpaceUiTextBatch;
use crate::graphics::scene::scene_renderer::ui::sdf_atlas::SdfAtlasCacheReport;
use crate::graphics::scene::scene_renderer::ui::sdf_render::ScreenSpaceUiSdfPrepareReport;
use crate::text::font::MissingGlyphDiagnosticsReport;
use crate::text::native_bitmap_atlas::{
    native_bitmap_atlas_handoff_for_report, NativeBitmapAtlasHandoff,
    NativeBitmapAtlasPrepareReport,
};
use crate::text::TextLayoutFallbackReport;

#[cfg(feature = "profiling")]
mod profile;
#[cfg(feature = "profiling")]
pub(super) use profile::record_text_prepare_profile;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ScreenSpaceUiTextPrepareReport {
    pub(super) input_auto_text_batch_count: usize,
    pub(super) input_native_text_batch_count: usize,
    pub(super) input_sdf_text_batch_count: usize,
    pub(super) resolved_native_text_batch_count: usize,
    pub(super) resolved_sdf_text_batch_count: usize,
    pub(super) auto_route: AutoTextRasterRouteFrameReport,
    pub(super) sdf_fallback: ScreenSpaceUiTextSdfFallbackReport,
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
    auto_texts: &[ScreenSpaceUiTextBatch],
    native_texts: &[ScreenSpaceUiTextBatch],
    sdf_texts: &[ScreenSpaceUiTextBatch],
    resolved_texts: &ResolvedScreenSpaceUiTextBatches,
    sdf_fallback: ScreenSpaceUiTextSdfFallbackReport,
    native_prepare: ScreenSpaceUiNativePrepareReport,
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
        input_auto_text_batch_count: auto_texts.len(),
        input_native_text_batch_count: native_texts.len(),
        input_sdf_text_batch_count: sdf_texts.len(),
        resolved_native_text_batch_count: resolved_texts.native_texts().len(),
        resolved_sdf_text_batch_count: resolved_texts.sdf_texts().len(),
        auto_route: resolved_texts.auto_route_report(),
        sdf_fallback,
        native_font_ids: native_prepare.font_ids,
        missing_glyphs,
        layout_fallbacks: resolved_texts.layout_fallback_report(),
        raster_upload,
        native_bitmap_atlas: native_prepare.bitmap_atlas,
        bitmap_atlas_renderer,
        sdf_atlas,
        sdf_generation,
        sdf_renderer,
    }
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
