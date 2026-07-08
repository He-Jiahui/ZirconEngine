use crate::core::framework::render::{RenderHybridGiPayloadSource, RenderStats};

use super::{record_bool, record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    record_probe_and_cache(store, stats);
    record_scene_and_surface_cache(store, stats);
    record_voxel_cache(store, stats);
    record_payload_source(store, stats);
}

fn record_probe_and_cache(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.hybrid_gi.active_probe_count",
        frame_index,
        stats.last_hybrid_gi_active_probe_count,
        &["render", "hybrid_gi", "probe"],
    );
    record_count(
        store,
        "render.hybrid_gi.requested_probe_count",
        frame_index,
        stats.last_hybrid_gi_requested_probe_count,
        &["render", "hybrid_gi", "probe", "request"],
    );
    record_count(
        store,
        "render.hybrid_gi.dirty_probe_count",
        frame_index,
        stats.last_hybrid_gi_dirty_probe_count,
        &["render", "hybrid_gi", "probe", "dirty"],
    );
    record_count(
        store,
        "render.hybrid_gi.cache_entry_count",
        frame_index,
        stats.last_hybrid_gi_cache_entry_count,
        &["render", "hybrid_gi", "cache"],
    );
    record_count(
        store,
        "render.hybrid_gi.resident_probe_count",
        frame_index,
        stats.last_hybrid_gi_resident_probe_count,
        &["render", "hybrid_gi", "probe", "resident"],
    );
    record_count(
        store,
        "render.hybrid_gi.pending_update_count",
        frame_index,
        stats.last_hybrid_gi_pending_update_count,
        &["render", "hybrid_gi", "update", "pending"],
    );
    record_count(
        store,
        "render.hybrid_gi.scheduled_trace_region_count",
        frame_index,
        stats.last_hybrid_gi_scheduled_trace_region_count,
        &["render", "hybrid_gi", "trace"],
    );
}

fn record_scene_and_surface_cache(store: &mut DiagnosticStore, stats: &RenderStats) {
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

fn record_voxel_cache(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.hybrid_gi.voxel.resident_clipmap_count",
        frame_index,
        stats.last_hybrid_gi_voxel_resident_clipmap_count,
        &["render", "hybrid_gi", "voxel", "resident"],
    );
    record_count(
        store,
        "render.hybrid_gi.voxel.dirty_clipmap_count",
        frame_index,
        stats.last_hybrid_gi_voxel_dirty_clipmap_count,
        &["render", "hybrid_gi", "voxel", "dirty"],
    );
    record_count(
        store,
        "render.hybrid_gi.voxel.invalidated_clipmap_count",
        frame_index,
        stats.last_hybrid_gi_voxel_invalidated_clipmap_count,
        &["render", "hybrid_gi", "voxel", "invalidation"],
    );
}

fn record_payload_source(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    let source = stats.last_hybrid_gi_payload_source;
    record_bool(
        store,
        "render.hybrid_gi.payload.source.none",
        frame_index,
        source == RenderHybridGiPayloadSource::None,
        &["render", "hybrid_gi", "payload", "source"],
    );
    record_bool(
        store,
        "render.hybrid_gi.payload.source.authored",
        frame_index,
        source == RenderHybridGiPayloadSource::Authored,
        &["render", "hybrid_gi", "payload", "source"],
    );
}
