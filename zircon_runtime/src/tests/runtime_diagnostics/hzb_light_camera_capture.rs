use crate::core::diagnostics::RuntimeDiagnosticsSnapshot;

use super::support::{assert_render_bool_series, assert_render_count_series};

pub(super) fn assert_hzb_light_camera_capture(snapshot: &RuntimeDiagnosticsSnapshot) {
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.mip_count",
        10.0,
        &["render", "hzb", "mip"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.graph_executed_pass_count",
        1.0,
        &["render", "hzb", "graph"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.hzb.occlusion.reported",
        true,
        &["render", "hzb", "occlusion"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.candidate_arg_count",
        6.0,
        &["render", "hzb", "occlusion", "candidate"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.candidate_instance_count",
        42.0,
        &["render", "hzb", "occlusion", "candidate"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.dispatch_group_count",
        2.0,
        &["render", "hzb", "occlusion", "dispatch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.dispatched_phase_count",
        1.0,
        &["render", "hzb", "occlusion", "dispatch"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.hzb.occlusion.history_available",
        true,
        &["render", "hzb", "occlusion", "history"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.hzb.occlusion.readback_available",
        true,
        &["render", "hzb", "occlusion", "readback"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.tested_arg_count",
        6.0,
        &["render", "hzb", "occlusion", "tested"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.tested_instance_count",
        42.0,
        &["render", "hzb", "occlusion", "tested"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.culled_arg_count",
        2.0,
        &["render", "hzb", "occlusion", "culled"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.culled_instance_count",
        18.0,
        &["render", "hzb", "occlusion", "culled"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.hzb.occlusion.indirect_args_readback_available",
        true,
        &["render", "hzb", "occlusion", "readback", "indirect_args"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.readback_arg_count",
        6.0,
        &["render", "hzb", "occlusion", "readback", "indirect_args"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.compacted_draw_count",
        4.0,
        &["render", "hzb", "occlusion", "readback", "indirect_args"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.zero_instance_arg_count",
        2.0,
        &["render", "hzb", "occlusion", "readback", "indirect_args"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.hzb.occlusion.remaining_instance_count",
        24.0,
        &["render", "hzb", "occlusion", "readback", "indirect_args"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.light_grid.reported",
        true,
        &["render", "light_grid"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.light_grid.light_count",
        9.0,
        &["render", "light_grid", "light"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.light_grid.tile_count",
        64.0,
        &["render", "light_grid", "tile"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.light_grid.zbin_count",
        32.0,
        &["render", "light_grid", "zbin"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.light_grid.non_empty_tile_count",
        11.0,
        &["render", "light_grid", "tile"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.light_grid.non_empty_zbin_count",
        7.0,
        &["render", "light_grid", "zbin"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.light_grid.non_empty_cluster_count",
        23.0,
        &["render", "light_grid", "cluster"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.light_grid.peak_lights_per_cluster",
        5.0,
        &["render", "light_grid", "cluster", "peak"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.light_grid.average_lights_per_cluster",
        0.375,
        &["render", "light_grid", "cluster", "average"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.primary_surface",
        false,
        &["render", "camera", "target", "primary_surface"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.headless",
        true,
        &["render", "camera", "target", "headless"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.texture",
        false,
        &["render", "camera", "target", "texture"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.primary_width",
        1280.0,
        &["render", "camera", "target", "primary"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.primary_height",
        720.0,
        &["render", "camera", "target", "primary"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.resolved_width",
        640.0,
        &["render", "camera", "target", "resolved"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.resolved_height",
        360.0,
        &["render", "camera", "target", "resolved"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.effective_view_width",
        320.0,
        &["render", "camera", "target", "effective_view"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.effective_view_height",
        180.0,
        &["render", "camera", "target", "effective_view"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.effective_render_width",
        160.0,
        &["render", "camera", "target", "effective_render"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.effective_render_height",
        90.0,
        &["render", "camera", "target", "effective_render"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.graph_import.not_requested",
        true,
        &["render", "camera", "target", "graph_import"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.graph_import.ready_for_direct_import",
        false,
        &["render", "camera", "target", "graph_import", "ready"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.graph_import.requires_conversion_writeback",
        false,
        &["render", "camera", "target", "graph_import", "conversion"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.graph_import.blocked_format_mismatch",
        false,
        &["render", "camera", "target", "graph_import", "blocked"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.graph_import.direct_import_count",
        0.0,
        &["render", "camera", "target", "graph_import"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.graph_import.conversion_writeback_count",
        0.0,
        &["render", "camera", "target", "graph_import"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.writeback.not_requested",
        true,
        &["render", "camera", "target", "writeback"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.writeback.copied",
        false,
        &["render", "camera", "target", "writeback", "copied"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.writeback.ready_for_conversion",
        false,
        &["render", "camera", "target", "writeback", "ready"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.writeback.converted",
        false,
        &["render", "camera", "target", "writeback", "converted"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.writeback.blocked_format_mismatch",
        false,
        &["render", "camera", "target", "writeback", "blocked"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.writeback.copy_count",
        0.0,
        &["render", "camera", "target", "writeback"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.writeback.converted_count",
        0.0,
        &["render", "camera", "target", "writeback"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.camera.target.writeback.debug_marker_emitted",
        false,
        &["render", "camera", "target", "writeback", "debug_marker"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.writeback.width",
        0.0,
        &["render", "camera", "target", "writeback", "extent"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.camera.target.writeback.height",
        0.0,
        &["render", "camera", "target", "writeback", "extent"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capture.source.none",
        true,
        &["render", "capture", "source"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.capture.source.framework_offscreen",
        false,
        &["render", "capture", "source", "framework_offscreen"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.capture.width",
        0.0,
        &["render", "capture", "extent"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.capture.height",
        0.0,
        &["render", "capture", "extent"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.invalidated.no_previous_frame",
        false,
        &["history", "invalidation", "no_previous_frame"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.invalidated.frame_inputs_changed",
        false,
        &["history", "invalidation", "frame_inputs_changed"],
    );
    assert_render_bool_series(
        &snapshot.store,
        "render.history.invalidated.render_size_changed",
        true,
        &["history", "invalidation", "render_size_changed"],
    );
}
