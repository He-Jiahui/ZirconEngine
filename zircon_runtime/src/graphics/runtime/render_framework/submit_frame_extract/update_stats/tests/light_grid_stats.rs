use super::update_light_grid_stats;
use crate::core::framework::render::RenderStats;
use crate::graphics::scene::RenderGraphLightGridReport;

#[test]
fn update_light_grid_stats_records_latest_grid_report() {
    let mut stats = RenderStats::default();
    let report = RenderGraphLightGridReport {
        light_count: 9,
        tile_count: 64,
        zbin_count: 32,
        non_empty_tile_count: 11,
        non_empty_zbin_count: 7,
        non_empty_cluster_count: 23,
        peak_lights_per_cluster: 5,
        average_lights_per_cluster_milli: 375,
    };

    update_light_grid_stats(&mut stats, Some(report));

    assert!(stats.last_light_grid_reported);
    assert_eq!(stats.last_light_grid_light_count, 9);
    assert_eq!(stats.last_light_grid_tile_count, 64);
    assert_eq!(stats.last_light_grid_zbin_count, 32);
    assert_eq!(stats.last_light_grid_non_empty_tile_count, 11);
    assert_eq!(stats.last_light_grid_non_empty_zbin_count, 7);
    assert_eq!(stats.last_light_grid_non_empty_cluster_count, 23);
    assert_eq!(stats.last_light_grid_peak_lights_per_cluster, 5);
    assert_eq!(stats.last_light_grid_average_lights_per_cluster_milli, 375);
}

#[test]
fn update_light_grid_stats_resets_when_no_report() {
    let mut stats = RenderStats {
        last_light_grid_reported: true,
        last_light_grid_light_count: 9,
        last_light_grid_tile_count: 64,
        last_light_grid_zbin_count: 32,
        last_light_grid_non_empty_tile_count: 11,
        last_light_grid_non_empty_zbin_count: 7,
        last_light_grid_non_empty_cluster_count: 23,
        last_light_grid_peak_lights_per_cluster: 5,
        last_light_grid_average_lights_per_cluster_milli: 375,
        ..RenderStats::default()
    };

    update_light_grid_stats(&mut stats, None);

    assert!(!stats.last_light_grid_reported);
    assert_eq!(stats.last_light_grid_light_count, 0);
    assert_eq!(stats.last_light_grid_tile_count, 0);
    assert_eq!(stats.last_light_grid_zbin_count, 0);
    assert_eq!(stats.last_light_grid_non_empty_tile_count, 0);
    assert_eq!(stats.last_light_grid_non_empty_zbin_count, 0);
    assert_eq!(stats.last_light_grid_non_empty_cluster_count, 0);
    assert_eq!(stats.last_light_grid_peak_lights_per_cluster, 0);
    assert_eq!(stats.last_light_grid_average_lights_per_cluster_milli, 0);
}
