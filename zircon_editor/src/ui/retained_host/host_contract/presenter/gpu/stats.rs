use zircon_runtime::rhi::{UiSurfacePresentStats, UiSurfacePresenter};

use super::GpuChromePresenter;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

pub(super) fn record_present_stats<P: UiSurfacePresenter>(
    presenter: &mut GpuChromePresenter<P>,
    stats: &UiSurfacePresentStats,
    region_present: bool,
) {
    presenter.last_upload_bytes = stats.image_upload_bytes;
    presenter.last_draw_calls = stats.draw_calls;
    record_current_ui_perf_counter(
        UiPerfCounter::GpuUploadBytes,
        stats.image_upload_bytes as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuImageUploadWrites,
        stats.image_upload_write_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuImageCacheKeyAllocations,
        stats.image_cache_key_allocation_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuImageCachePruneVisits,
        stats.image_cache_prune_visit_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuImageCacheAdmissionRejects,
        stats.image_cache_admission_reject_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuImageInvalidPayloads,
        stats.image_invalid_payload_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuImageCacheResidentBytes,
        stats.image_cache_resident_bytes as f64,
    );
    record_current_ui_perf_counter(UiPerfCounter::GpuDrawCalls, stats.draw_calls as f64);
    record_current_ui_perf_counter(
        UiPerfCounter::GpuCompiledDrawCalls,
        stats.compiled_draw_calls as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuRenderPasses,
        stats.render_pass_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuVisibleCommands,
        stats.visible_command_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuVisibleCommandPayloadBytes,
        stats.visible_command_payload_bytes as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuVisibleDrawItems,
        stats.visible_draw_item_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuCompiledDrawItems,
        stats.compiled_visible_draw_item_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuCommandVisibilityScans,
        stats.command_visibility_scan_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuCommandStatsCacheHits,
        stats.command_stats_cache_hit_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuSolidVertices,
        stats.solid_vertex_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuCompiledSolidVertices,
        stats.compiled_solid_vertex_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuImageVertices,
        stats.image_vertex_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuCompiledImageVertices,
        stats.compiled_image_vertex_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuBatchLayers,
        stats.batch_layer_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuCompiledBatchLayers,
        stats.compiled_batch_layer_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuBatchDependencies,
        stats.batch_dependency_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuCompiledBatchDependencies,
        stats.compiled_batch_dependency_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuBatchMerges,
        stats.batch_merge_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuCompiledBatchMerges,
        stats.compiled_batch_merge_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuOverlapCandidates,
        stats.overlap_candidate_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuBatchPlanBuilds,
        stats.batch_plan_build_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuBatchPlanCacheHits,
        stats.batch_plan_cache_hit_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuVertexBufferCreates,
        stats.vertex_buffer_create_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuVertexUploadBytes,
        stats.vertex_upload_bytes as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuRetainedCacheCopyBytes,
        stats.retained_cache_copy_bytes as f64,
    );
    record_current_ui_perf_counter(UiPerfCounter::GpuTextShapes, stats.text_shape_count as f64);
    record_current_ui_perf_counter(
        UiPerfCounter::GpuTextRendererBuilds,
        stats.text_renderer_build_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuTextRendererCacheHits,
        stats.text_renderer_cache_hit_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuTextPrepareFailures,
        stats.text_prepare_failure_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuImagePrepareCommandVisits,
        stats.image_prepare_command_visit_count as f64,
    );
    record_current_ui_perf_counter(
        UiPerfCounter::GpuImagePrepareCacheHits,
        stats.image_prepare_cache_hit_count as f64,
    );
    if region_present {
        record_current_ui_perf_counter(UiPerfCounter::ChromeCommandPatchCount, 1.0);
    } else {
        record_current_ui_perf_counter(UiPerfCounter::ChromeCommandFullRebuildCount, 1.0);
    }
}
