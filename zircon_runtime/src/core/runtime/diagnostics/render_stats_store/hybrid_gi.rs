mod global_sdf;
mod payload_source;
mod probe_cache;
mod scene_surface_cache;
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
