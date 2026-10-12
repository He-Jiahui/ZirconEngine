use crate::core::framework::render::RenderStats;

use super::super::{record_count, DiagnosticStore};

/// 区分体素级联的驻留、脏化与失效，判断重建压力来自可见需求还是历史作废。
pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
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
