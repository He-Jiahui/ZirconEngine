use crate::core::diagnostics::DiagnosticStore;
use crate::core::framework::render::RenderStats;

use super::super::record;
use super::assert_series;

#[test]
fn render_product_diagnostics_record_visibility_stats() {
    let mut store = DiagnosticStore::default();
    let stats = RenderStats {
        submitted_frames: 12,
        last_visibility_view_count: 2,
        last_visibility_input_count: 8,
        last_visibility_layer_filtered_count: 1,
        last_visibility_frustum_culled_count: 3,
        last_visibility_occlusion_culled_count: 1,
        last_visibility_visible_count: 3,
        last_visibility_static_index_full_rebuild_count: 0,
        last_visibility_static_index_incremental_update_count: 1,
        last_visibility_static_index_inserted_count: 2,
        last_visibility_static_index_updated_count: 3,
        last_visibility_static_index_removed_count: 4,
        last_visibility_static_index_indexed_entity_count: 10,
        last_visibility_static_index_occupied_cell_count: 7,
        last_visibility_static_index_main_view_prefilter_used: true,
        last_visibility_static_index_main_view_static_input_count: 12,
        last_visibility_static_index_main_view_static_candidate_count: 5,
        ..RenderStats::default()
    };

    record(&mut store, &stats);

    assert_series(&store, "render.visibility.view_count", 2.0, "count");
    assert_series(&store, "render.visibility.input_count", 8.0, "count");
    assert_series(
        &store,
        "render.visibility.layer_filtered_count",
        1.0,
        "count",
    );
    assert_series(
        &store,
        "render.visibility.frustum_culled_count",
        3.0,
        "count",
    );
    assert_series(
        &store,
        "render.visibility.occlusion_culled_count",
        1.0,
        "count",
    );
    assert_series(&store, "render.visibility.visible_count", 3.0, "count");
    assert_series(
        &store,
        "render.visibility.static_index.full_rebuild_count",
        0.0,
        "count",
    );
    assert_series(
        &store,
        "render.visibility.static_index.incremental_update_count",
        1.0,
        "count",
    );
    assert_series(
        &store,
        "render.visibility.static_index.inserted_count",
        2.0,
        "count",
    );
    assert_series(
        &store,
        "render.visibility.static_index.updated_count",
        3.0,
        "count",
    );
    assert_series(
        &store,
        "render.visibility.static_index.removed_count",
        4.0,
        "count",
    );
    assert_series(
        &store,
        "render.visibility.static_index.indexed_entity_count",
        10.0,
        "count",
    );
    assert_series(
        &store,
        "render.visibility.static_index.occupied_cell_count",
        7.0,
        "count",
    );
    assert_series(
        &store,
        "render.visibility.static_index.main_view_prefilter_used",
        1.0,
        "bool",
    );
    assert_series(
        &store,
        "render.visibility.static_index.main_view_static_input_count",
        12.0,
        "count",
    );
    assert_series(
        &store,
        "render.visibility.static_index.main_view_static_candidate_count",
        5.0,
        "count",
    );
}

#[test]
fn render_product_diagnostics_record_hzb_stats() {
    let mut store = DiagnosticStore::default();
    let stats = RenderStats {
        submitted_frames: 12,
        last_hzb_mip_count: 10,
        last_hzb_graph_executed_pass_count: 1,
        last_hzb_occlusion_reported: true,
        last_hzb_occlusion_candidate_arg_count: 6,
        last_hzb_occlusion_candidate_instance_count: 42,
        last_hzb_occlusion_dispatch_group_count: 2,
        last_hzb_occlusion_dispatched_phase_count: 1,
        last_hzb_occlusion_history_available: true,
        last_hzb_occlusion_readback_available: true,
        last_hzb_occlusion_readback_pending_count: 3,
        last_hzb_occlusion_readback_dropped_count: 1,
        last_hzb_occlusion_readback_oldest_pending_age_frames: Some(4),
        last_hzb_occlusion_tested_arg_count: 6,
        last_hzb_occlusion_tested_instance_count: 42,
        last_hzb_occlusion_culled_arg_count: 2,
        last_hzb_occlusion_culled_instance_count: 18,
        last_hzb_occlusion_indirect_args_readback_available: true,
        last_hzb_occlusion_readback_arg_count: 6,
        last_hzb_occlusion_compacted_draw_count: 4,
        last_hzb_occlusion_zero_instance_arg_count: 2,
        last_hzb_occlusion_remaining_instance_count: 24,
        ..RenderStats::default()
    };

    record(&mut store, &stats);

    assert_series(&store, "render.hzb.mip_count", 10.0, "count");
    assert_series(&store, "render.hzb.graph_executed_pass_count", 1.0, "count");
    assert_series(&store, "render.hzb.occlusion.reported", 1.0, "bool");
    assert_series(
        &store,
        "render.hzb.occlusion.candidate_arg_count",
        6.0,
        "count",
    );
    assert_series(
        &store,
        "render.hzb.occlusion.candidate_instance_count",
        42.0,
        "count",
    );
    assert_series(
        &store,
        "render.hzb.occlusion.dispatch_group_count",
        2.0,
        "count",
    );
    assert_series(
        &store,
        "render.hzb.occlusion.dispatched_phase_count",
        1.0,
        "count",
    );
    assert_series(
        &store,
        "render.hzb.occlusion.history_available",
        1.0,
        "bool",
    );
    assert_series(
        &store,
        "render.hzb.occlusion.readback_available",
        1.0,
        "bool",
    );
    assert_series(
        &store,
        "render.hzb.occlusion.tested_arg_count",
        6.0,
        "count",
    );
    assert_series(
        &store,
        "render.hzb.occlusion.tested_instance_count",
        42.0,
        "count",
    );
    assert_series(
        &store,
        "render.hzb.occlusion.culled_arg_count",
        2.0,
        "count",
    );
    assert_series(
        &store,
        "render.hzb.occlusion.culled_instance_count",
        18.0,
        "count",
    );
    assert_series(
        &store,
        "render.hzb.occlusion.indirect_args_readback_available",
        1.0,
        "bool",
    );
    assert_series(
        &store,
        "render.hzb.occlusion.readback_arg_count",
        6.0,
        "count",
    );
    assert_series(
        &store,
        "render.hzb.occlusion.compacted_draw_count",
        4.0,
        "count",
    );
    assert_series(
        &store,
        "render.hzb.occlusion.zero_instance_arg_count",
        2.0,
        "count",
    );
    assert_series(
        &store,
        "render.hzb.occlusion.remaining_instance_count",
        24.0,
        "count",
    );
}

#[test]
fn render_product_diagnostics_record_light_grid_stats() {
    let mut store = DiagnosticStore::default();
    let stats = RenderStats {
        submitted_frames: 12,
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

    record(&mut store, &stats);

    assert_series(&store, "render.light_grid.reported", 1.0, "bool");
    assert_series(&store, "render.light_grid.light_count", 9.0, "count");
    assert_series(&store, "render.light_grid.tile_count", 64.0, "count");
    assert_series(&store, "render.light_grid.zbin_count", 32.0, "count");
    assert_series(
        &store,
        "render.light_grid.non_empty_tile_count",
        11.0,
        "count",
    );
    assert_series(
        &store,
        "render.light_grid.non_empty_zbin_count",
        7.0,
        "count",
    );
    assert_series(
        &store,
        "render.light_grid.non_empty_cluster_count",
        23.0,
        "count",
    );
    assert_series(
        &store,
        "render.light_grid.peak_lights_per_cluster",
        5.0,
        "count",
    );
    assert_series(
        &store,
        "render.light_grid.average_lights_per_cluster",
        0.375,
        "count",
    );
    assert_series(
        &store,
        "render.hzb.occlusion.readback_pending_count",
        3.0,
        "count",
    );
    assert_series(
        &store,
        "render.hzb.occlusion.readback_dropped_count",
        1.0,
        "count",
    );
    assert_series(
        &store,
        "render.hzb.occlusion.readback_oldest_pending_age_frames",
        4.0,
        "count",
    );
}
