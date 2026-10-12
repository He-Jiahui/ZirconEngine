//! 混合全局光照的探针、场景表面、体素和 SDF 诊断在此汇合，共享提交帧索引。
#[path = "gi/global_sdf.rs"]
mod global_sdf;
#[path = "../gi/payload_source.rs"]
mod payload_source;
#[path = "gi/probe_cache.rs"]
mod probe_cache;
#[path = "../gi/scene_surface_cache.rs"]
mod scene_surface_cache;
#[path = "gi/voxel_cache.rs"]
mod voxel_cache;

use crate::core::framework::render::RenderStats;

use super::DiagnosticStore;

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    probe_cache::record(store, stats);
    scene_surface_cache::record(store, stats);
    voxel_cache::record(store, stats);
    global_sdf::record(store, stats);
    payload_source::record(store, stats);
}
