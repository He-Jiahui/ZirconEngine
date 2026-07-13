use super::super::sources::HotspotInventorySources;

pub(super) fn assert_extract_evidence(sources: &HotspotInventorySources) {
    for required_extract_anchor in [
        "headless_session_tick_publishes_ecs_frame_diagnostics",
        "headless_session_capture_records_frame_extract_diagnostics",
        "frame_extract_rebuild_skips_unchanged_entities",
        "EXTRACT_REBUILD_CLONES_DIAGNOSTIC",
        "EXTRACT_OUTPUT_BYTES_DIAGNOSTIC",
        "EXTRACT_CACHE_HITS_DIAGNOSTIC",
        "EXTRACT_CACHE_MISSES_DIAGNOSTIC",
        "rebuilds.history[1].value, 0.0",
        "cache_hits.history[1].value, 1.0",
        "cache_misses.history[0].value, 1.0",
        "unchanged headless capture should reuse the cached extract",
        "frame_extract_rebuilds_after_scene_change",
        "scene mutations should invalidate the dynamic-session extract cache",
        "output_bytes.history[0].value, output_bytes.history[1].value",
    ] {
        assert!(
            sources.session_tests.contains(required_extract_anchor),
            "extract evidence should retain `{required_extract_anchor}`"
        );
    }

    for required_extract_cache_anchor in [
        "pub(super) struct RuntimeFrameExtractCache",
        "struct RuntimeFrameExtractCacheKey",
        "change_tick: world.read_change_tick()",
        "query_cache_revision: world.query_cache_revision()",
        "active_camera: world.active_camera()",
        "RuntimeFrameExtractCacheStatus::Rebuilt => 1",
        "RuntimeFrameExtractCacheStatus::Reused => 0",
    ] {
        assert!(
            sources
                .session_extract_cache
                .contains(required_extract_cache_anchor)
                || sources
                    .session_extract_stats
                    .contains(required_extract_cache_anchor),
            "extract cache path should retain `{required_extract_cache_anchor}`"
        );
    }
}
