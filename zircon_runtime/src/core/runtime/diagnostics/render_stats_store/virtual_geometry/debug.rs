use crate::core::framework::render::RenderStats;

use super::super::{record_bool, record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_bool(
        store,
        "render.virtual_geometry.forced_mip_present",
        frame_index,
        stats.last_virtual_geometry_forced_mip.is_some(),
        &["render", "virtual_geometry", "debug", "mip"],
    );
    record_count(
        store,
        "render.virtual_geometry.forced_mip_value",
        frame_index,
        stats
            .last_virtual_geometry_forced_mip
            .map_or(0, usize::from),
        &["render", "virtual_geometry", "debug", "mip"],
    );
    record_bool(
        store,
        "render.virtual_geometry.debug.freeze_cull",
        frame_index,
        stats.last_virtual_geometry_freeze_cull,
        &["render", "virtual_geometry", "debug"],
    );
    record_bool(
        store,
        "render.virtual_geometry.debug.visualize_bvh",
        frame_index,
        stats.last_virtual_geometry_visualize_bvh,
        &["render", "virtual_geometry", "debug"],
    );
    record_bool(
        store,
        "render.virtual_geometry.debug.visualize_visbuffer",
        frame_index,
        stats.last_virtual_geometry_visualize_visbuffer,
        &["render", "virtual_geometry", "debug"],
    );
    record_bool(
        store,
        "render.virtual_geometry.debug.print_leaf_clusters",
        frame_index,
        stats.last_virtual_geometry_print_leaf_clusters,
        &["render", "virtual_geometry", "debug"],
    );
}
