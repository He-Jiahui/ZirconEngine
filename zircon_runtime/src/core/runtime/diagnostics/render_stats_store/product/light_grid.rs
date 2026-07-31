use crate::core::framework::render::RenderStats;

use super::{DiagnosticStore, record_bool, record_count};
pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_bool(
        store,
        "render.light_grid.reported",
        frame_index,
        stats.last_light_grid_reported,
        &["render", "light_grid"],
    );
    record_count(
        store,
        "render.light_grid.light_count",
        frame_index,
        stats.last_light_grid_light_count,
        &["render", "light_grid", "light"],
    );
    record_count(
        store,
        "render.light_grid.tile_count",
        frame_index,
        stats.last_light_grid_tile_count,
        &["render", "light_grid", "tile"],
    );
    record_count(
        store,
        "render.light_grid.zbin_count",
        frame_index,
        stats.last_light_grid_zbin_count,
        &["render", "light_grid", "zbin"],
    );
    record_count(
        store,
        "render.light_grid.non_empty_tile_count",
        frame_index,
        stats.last_light_grid_non_empty_tile_count,
        &["render", "light_grid", "tile"],
    );
    record_count(
        store,
        "render.light_grid.non_empty_zbin_count",
        frame_index,
        stats.last_light_grid_non_empty_zbin_count,
        &["render", "light_grid", "zbin"],
    );
    record_count(
        store,
        "render.light_grid.non_empty_cluster_count",
        frame_index,
        stats.last_light_grid_non_empty_cluster_count,
        &["render", "light_grid", "cluster"],
    );
    record_count(
        store,
        "render.light_grid.peak_lights_per_cluster",
        frame_index,
        stats.last_light_grid_peak_lights_per_cluster,
        &["render", "light_grid", "cluster", "peak"],
    );
    store.record_static(
        "render.light_grid.average_lights_per_cluster",
        frame_index,
        stats.last_light_grid_average_lights_per_cluster_milli as f64 / 1000.0,
        Some("count"),
        &["render", "light_grid", "cluster", "average"],
    );
}
