use zircon_runtime::rhi::{UiSurfacePresentStats, UiSurfacePresenter};

use super::GpuChromePresenter;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter_batch, UiPerfCounter};

pub(super) fn record_present_stats<P: UiSurfacePresenter>(
    presenter: &mut GpuChromePresenter<P>,
    stats: &UiSurfacePresentStats,
    region_present: bool,
) {
    presenter.last_upload_bytes = stats.image_upload_bytes;
    presenter.last_draw_calls = stats.draw_calls;
    record_current_ui_perf_counter_batch(|counters| {
        append_present_stats(counters, stats, region_present)
    });
}

fn append_present_stats(
    counters: &mut Vec<(UiPerfCounter, f64)>,
    stats: &UiSurfacePresentStats,
    region_present: bool,
) {
    let mut record = |counter, value| counters.push((counter, value));
    record(
        UiPerfCounter::GpuUploadBytes,
        stats.image_upload_bytes as f64,
    );
    record(
        UiPerfCounter::GpuImageUploadWrites,
        stats.image_upload_write_count as f64,
    );
    record(
        UiPerfCounter::GpuImageSharedResolves,
        stats.image_shared_resolve_count as f64,
    );
    record(
        UiPerfCounter::GpuImageSharedUploadWrites,
        stats.image_shared_upload_write_count as f64,
    );
    record(
        UiPerfCounter::GpuImageSharedUploadBytes,
        stats.image_shared_upload_bytes as f64,
    );
    record(
        UiPerfCounter::GpuImageSharedResidentBytes,
        stats.image_shared_resident_bytes as f64,
    );
    record(
        UiPerfCounter::GpuImageCacheKeyAllocations,
        stats.image_cache_key_allocation_count as f64,
    );
    record(
        UiPerfCounter::GpuImageCachePruneVisits,
        stats.image_cache_prune_visit_count as f64,
    );
    record(
        UiPerfCounter::GpuImageCacheAdmissionRejects,
        stats.image_cache_admission_reject_count as f64,
    );
    record(
        UiPerfCounter::GpuImageInvalidPayloads,
        stats.image_invalid_payload_count as f64,
    );
    record(
        UiPerfCounter::GpuImageCacheResidentBytes,
        stats.image_cache_resident_bytes as f64,
    );
    record(UiPerfCounter::GpuDrawCalls, stats.draw_calls as f64);
    record(
        UiPerfCounter::GpuCompiledDrawCalls,
        stats.compiled_draw_calls as f64,
    );
    record(
        UiPerfCounter::GpuRenderPasses,
        stats.render_pass_count as f64,
    );
    if stats.gpu_timestamp_supported {
        record(UiPerfCounter::GpuTimestampSupportedPresentCount, 1.0);
    }
    if let Some(gpu_time_us) = stats.gpu_time_us {
        record(UiPerfCounter::GpuTimeUs, gpu_time_us as f64);
        record(
            UiPerfCounter::GpuProfileLatencyFrames,
            stats.gpu_profile_latency_frames as f64,
        );
    }
    record(
        UiPerfCounter::GpuVisibleCommands,
        stats.visible_command_count as f64,
    );
    record(
        UiPerfCounter::GpuVisibleCommandPayloadBytes,
        stats.visible_command_payload_bytes as f64,
    );
    record(
        UiPerfCounter::GpuVisibleCommandStyles,
        stats.visible_command_style_count as f64,
    );
    record(
        UiPerfCounter::GpuVisibleDrawItems,
        stats.visible_draw_item_count as f64,
    );
    record(
        UiPerfCounter::GpuCompiledDrawItems,
        stats.compiled_visible_draw_item_count as f64,
    );
    record(
        UiPerfCounter::GpuCommandVisibilityScans,
        stats.command_visibility_scan_count as f64,
    );
    record(
        UiPerfCounter::GpuCommandStatsCacheHits,
        stats.command_stats_cache_hit_count as f64,
    );
    record(
        UiPerfCounter::GpuSolidVertices,
        stats.solid_vertex_count as f64,
    );
    record(
        UiPerfCounter::GpuCompiledSolidVertices,
        stats.compiled_solid_vertex_count as f64,
    );
    record(
        UiPerfCounter::GpuSolidInstances,
        stats.solid_instance_count as f64,
    );
    record(
        UiPerfCounter::GpuCompiledSolidInstances,
        stats.compiled_solid_instance_count as f64,
    );
    record(
        UiPerfCounter::GpuImageVertices,
        stats.image_vertex_count as f64,
    );
    record(
        UiPerfCounter::GpuCompiledImageVertices,
        stats.compiled_image_vertex_count as f64,
    );
    record(
        UiPerfCounter::GpuBatchLayers,
        stats.batch_layer_count as f64,
    );
    record(
        UiPerfCounter::GpuCompiledBatchLayers,
        stats.compiled_batch_layer_count as f64,
    );
    record(
        UiPerfCounter::GpuBatchDependencies,
        stats.batch_dependency_count as f64,
    );
    record(
        UiPerfCounter::GpuCompiledBatchDependencies,
        stats.compiled_batch_dependency_count as f64,
    );
    record(
        UiPerfCounter::GpuBatchMerges,
        stats.batch_merge_count as f64,
    );
    record(
        UiPerfCounter::GpuCompiledBatchMerges,
        stats.compiled_batch_merge_count as f64,
    );
    record(
        UiPerfCounter::GpuOverlapCandidates,
        stats.overlap_candidate_count as f64,
    );
    record(
        UiPerfCounter::GpuBatchPlanBuilds,
        stats.batch_plan_build_count as f64,
    );
    record(
        UiPerfCounter::GpuBatchPlanCacheHits,
        stats.batch_plan_cache_hit_count as f64,
    );
    record(
        UiPerfCounter::GpuVertexBufferCreates,
        stats.vertex_buffer_create_count as f64,
    );
    record(
        UiPerfCounter::GpuVertexUploadBytes,
        stats.vertex_upload_bytes as f64,
    );
    record(
        UiPerfCounter::GpuRetainedCacheCopyBytes,
        stats.retained_cache_copy_bytes as f64,
    );
    record(UiPerfCounter::GpuTextShapes, stats.text_shape_count as f64);
    record(
        UiPerfCounter::GpuTextRendererBuilds,
        stats.text_renderer_build_count as f64,
    );
    record(
        UiPerfCounter::GpuTextRendererCacheHits,
        stats.text_renderer_cache_hit_count as f64,
    );
    record(
        UiPerfCounter::GpuTextPrepareFailures,
        stats.text_prepare_failure_count as f64,
    );
    record(
        UiPerfCounter::GpuImagePrepareCommandVisits,
        stats.image_prepare_command_visit_count as f64,
    );
    record(
        UiPerfCounter::GpuImagePrepareCacheHits,
        stats.image_prepare_cache_hit_count as f64,
    );
    record(
        if region_present {
            UiPerfCounter::ChromeCommandPatchCount
        } else {
            UiPerfCounter::ChromeCommandFullRebuildCount
        },
        1.0,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn present_stats_batch_preserves_region_and_optional_counter_semantics() {
        let mut counters = Vec::new();
        append_present_stats(&mut counters, &UiSurfacePresentStats::default(), true);

        assert_eq!(counters.len(), 46);
        assert_eq!(
            counters.last(),
            Some(&(UiPerfCounter::ChromeCommandPatchCount, 1.0))
        );
        assert!(!counters
            .iter()
            .any(|(counter, _)| *counter == UiPerfCounter::GpuTimestampSupportedPresentCount));
        assert!(!counters
            .iter()
            .any(|(counter, _)| *counter == UiPerfCounter::GpuTimeUs));
        assert!(!counters
            .iter()
            .any(|(counter, _)| *counter == UiPerfCounter::GpuProfileLatencyFrames));
    }

    #[test]
    fn present_stats_batch_includes_supported_gpu_timing_once() {
        let mut stats = UiSurfacePresentStats::default();
        stats.gpu_timestamp_supported = true;
        stats.gpu_time_us = Some(125);
        stats.gpu_profile_latency_frames = 2;
        let mut counters = Vec::new();
        append_present_stats(&mut counters, &stats, false);

        assert_eq!(counters.len(), 49);
        assert_eq!(
            counters.last(),
            Some(&(UiPerfCounter::ChromeCommandFullRebuildCount, 1.0))
        );
        assert_eq!(
            counters
                .iter()
                .filter(|(counter, _)| {
                    *counter == UiPerfCounter::GpuTimestampSupportedPresentCount
                })
                .count(),
            1
        );
        assert!(counters.contains(&(UiPerfCounter::GpuTimeUs, stats.gpu_time_us.unwrap() as f64)));
        assert!(counters.contains(&(
            UiPerfCounter::GpuProfileLatencyFrames,
            stats.gpu_profile_latency_frames as f64,
        )));
    }
}
