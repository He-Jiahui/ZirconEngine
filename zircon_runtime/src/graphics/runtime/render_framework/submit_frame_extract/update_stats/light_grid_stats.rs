use crate::core::framework::render::RenderStats;
use crate::graphics::scene::RenderGraphLightGridReport;

pub(super) fn update_light_grid_stats(
    stats: &mut RenderStats,
    report: Option<RenderGraphLightGridReport>,
) {
    let Some(report) = report else {
        stats.last_light_grid_reported = false;
        stats.last_light_grid_light_count = 0;
        stats.last_light_grid_tile_count = 0;
        stats.last_light_grid_zbin_count = 0;
        stats.last_light_grid_non_empty_tile_count = 0;
        stats.last_light_grid_non_empty_zbin_count = 0;
        stats.last_light_grid_non_empty_cluster_count = 0;
        stats.last_light_grid_peak_lights_per_cluster = 0;
        stats.last_light_grid_average_lights_per_cluster_milli = 0;
        return;
    };

    stats.last_light_grid_reported = true;
    stats.last_light_grid_light_count = report.light_count;
    stats.last_light_grid_tile_count = report.tile_count;
    stats.last_light_grid_zbin_count = report.zbin_count;
    stats.last_light_grid_non_empty_tile_count = report.non_empty_tile_count;
    stats.last_light_grid_non_empty_zbin_count = report.non_empty_zbin_count;
    stats.last_light_grid_non_empty_cluster_count = report.non_empty_cluster_count;
    stats.last_light_grid_peak_lights_per_cluster = report.peak_lights_per_cluster;
    stats.last_light_grid_average_lights_per_cluster_milli =
        report.average_lights_per_cluster_milli;
}

#[cfg(test)]
#[path = "tests/light_grid_stats.rs"]
mod tests;
