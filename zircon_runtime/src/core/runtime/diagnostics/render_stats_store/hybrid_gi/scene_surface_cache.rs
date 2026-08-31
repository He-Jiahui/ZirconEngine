use crate::core::framework::render::{RenderHybridGiRadianceCacheGpuStage, RenderStats};

use super::super::{record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.hybrid_gi.scene.card_count",
        frame_index,
        stats.last_hybrid_gi_scene_card_count,
        &["render", "hybrid_gi", "scene"],
    );
    record_count(
        store,
        "render.hybrid_gi.scene.screen_probe_count",
        frame_index,
        stats.last_hybrid_gi_scene_screen_probe_count,
        &["render", "hybrid_gi", "scene", "screen_probe"],
    );
    record_count(
        store,
        "render.hybrid_gi.scene.radiance_cache_entry_count",
        frame_index,
        stats.last_hybrid_gi_scene_radiance_cache_entry_count,
        &["render", "hybrid_gi", "scene", "radiance_cache"],
    );
    record_count(
        store,
        "render.hybrid_gi.radiance_cache.resident_probe_count",
        frame_index,
        stats.last_hybrid_gi_radiance_cache_resident_probe_count,
        &["render", "hybrid_gi", "radiance_cache", "resident"],
    );
    record_count(
        store,
        "render.hybrid_gi.radiance_cache.update_probe_count",
        frame_index,
        stats.last_hybrid_gi_radiance_cache_update_probe_count,
        &["render", "hybrid_gi", "radiance_cache", "update"],
    );
    record_count(
        store,
        "render.hybrid_gi.radiance_cache.truncated_demand_count",
        frame_index,
        stats.last_hybrid_gi_radiance_cache_truncated_demand_count,
        &["render", "hybrid_gi", "radiance_cache", "budget"],
    );
    record_count(
        store,
        "render.hybrid_gi.radiance_cache.generation",
        frame_index,
        usize::try_from(stats.last_hybrid_gi_radiance_cache_generation).unwrap_or(usize::MAX),
        &["render", "hybrid_gi", "radiance_cache", "generation"],
    );
    record_count(
        store,
        "render.hybrid_gi.radiance_cache.scroll_count",
        frame_index,
        usize::try_from(stats.last_hybrid_gi_radiance_cache_scroll_count).unwrap_or(usize::MAX),
        &["render", "hybrid_gi", "radiance_cache", "scroll"],
    );
    record_count(
        store,
        "render.hybrid_gi.radiance_cache.history_clear_count",
        frame_index,
        usize::try_from(stats.last_hybrid_gi_radiance_cache_history_clear_count)
            .unwrap_or(usize::MAX),
        &["render", "hybrid_gi", "radiance_cache", "history"],
    );
    for (stage, dispatch_count) in RenderHybridGiRadianceCacheGpuStage::ALL.into_iter().zip(
        stats
            .last_hybrid_gi_radiance_cache_gpu_stage_dispatch_counts
            .iter()
            .copied(),
    ) {
        record_count(
            store,
            match stage {
                RenderHybridGiRadianceCacheGpuStage::Mark => {
                    "render.hybrid_gi.radiance_cache.gpu_dispatch.mark"
                }
                RenderHybridGiRadianceCacheGpuStage::Allocate => {
                    "render.hybrid_gi.radiance_cache.gpu_dispatch.allocate"
                }
                RenderHybridGiRadianceCacheGpuStage::Trace => {
                    "render.hybrid_gi.radiance_cache.gpu_dispatch.trace"
                }
                RenderHybridGiRadianceCacheGpuStage::Filter => {
                    "render.hybrid_gi.radiance_cache.gpu_dispatch.filter"
                }
                RenderHybridGiRadianceCacheGpuStage::BorderMip => {
                    "render.hybrid_gi.radiance_cache.gpu_dispatch.border_mip"
                }
                RenderHybridGiRadianceCacheGpuStage::Consume => {
                    "render.hybrid_gi.radiance_cache.gpu_dispatch.consume"
                }
            },
            frame_index,
            usize::try_from(dispatch_count).unwrap_or(usize::MAX),
            &["render", "hybrid_gi", "radiance_cache", "gpu_dispatch"],
        );
    }
    record_count(
        store,
        "render.hybrid_gi.surface_cache.resident_page_count",
        frame_index,
        stats.last_hybrid_gi_surface_cache_resident_page_count,
        &["render", "hybrid_gi", "surface_cache", "resident"],
    );
    record_count(
        store,
        "render.hybrid_gi.surface_cache.dirty_page_count",
        frame_index,
        stats.last_hybrid_gi_surface_cache_dirty_page_count,
        &["render", "hybrid_gi", "surface_cache", "dirty"],
    );
    record_count(
        store,
        "render.hybrid_gi.surface_cache.feedback_card_count",
        frame_index,
        stats.last_hybrid_gi_surface_cache_feedback_card_count,
        &["render", "hybrid_gi", "surface_cache", "feedback"],
    );
    record_count(
        store,
        "render.hybrid_gi.surface_cache.capture_slot_count",
        frame_index,
        stats.last_hybrid_gi_surface_cache_capture_slot_count,
        &["render", "hybrid_gi", "surface_cache", "capture"],
    );
    record_count(
        store,
        "render.hybrid_gi.surface_cache.invalidated_page_count",
        frame_index,
        stats.last_hybrid_gi_surface_cache_invalidated_page_count,
        &["render", "hybrid_gi", "surface_cache", "invalidation"],
    );
    record_count(
        store,
        "render.hybrid_gi.surface_cache.depth_sample_count",
        frame_index,
        stats.last_hybrid_gi_surface_cache_depth_sample_count,
        &["render", "hybrid_gi", "surface_cache", "depth"],
    );
    record_count(
        store,
        "render.hybrid_gi.probe_trace.tile_count",
        frame_index,
        stats.last_hybrid_gi_probe_trace_tile_count,
        &["render", "hybrid_gi", "probe_trace", "tile"],
    );
    for (axis, group_count) in stats
        .last_hybrid_gi_probe_trace_dispatch_group_count
        .iter()
        .copied()
        .enumerate()
    {
        record_count(
            store,
            match axis {
                0 => "render.hybrid_gi.probe_trace.dispatch_group_count.x",
                1 => "render.hybrid_gi.probe_trace.dispatch_group_count.y",
                _ => "render.hybrid_gi.probe_trace.dispatch_group_count.z",
            },
            frame_index,
            group_count,
            &["render", "hybrid_gi", "probe_trace", "dispatch"],
        );
    }
}
