use crate::core::framework::render::RenderStats;

use super::super::{record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
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
