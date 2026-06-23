use crate::core::diagnostics::RuntimeDiagnosticsSnapshot;

use super::support::{
    assert_render_bool_series, assert_render_count_series, assert_render_microsecond_series,
};

pub(super) fn assert_graph_execution(snapshot: &RuntimeDiagnosticsSnapshot) {
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.alias.texture_logical_count",
        3.0,
        &["graph", "execution", "resource", "alias", "texture"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.alias.texture_alias_count",
        2.0,
        &["graph", "execution", "resource", "alias", "texture"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.alias.texture_backing_count",
        2.0,
        &[
            "graph",
            "execution",
            "resource",
            "alias",
            "texture",
            "backing",
        ],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.alias.buffer_logical_count",
        3.0,
        &["graph", "execution", "resource", "alias", "buffer"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.alias.buffer_alias_count",
        2.0,
        &["graph", "execution", "resource", "alias", "buffer"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.alias.buffer_backing_count",
        2.0,
        &[
            "graph",
            "execution",
            "resource",
            "alias",
            "buffer",
            "backing",
        ],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.coverage.planned_live_pass_count",
        14.0,
        &["graph", "execution", "coverage"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.coverage.executed_pass_count",
        14.0,
        &["graph", "execution", "coverage"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.coverage.matched_planned_pass_count",
        14.0,
        &["graph", "execution", "coverage"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.coverage.missing_planned_pass_count",
        0.0,
        &["graph", "execution", "coverage"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.coverage.unexpected_executed_pass_count",
        0.0,
        &["graph", "execution", "coverage"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.coverage.duplicate_executed_pass_count",
        0.0,
        &["graph", "execution", "coverage"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.stage.staged_pass_count",
        14.0,
        &["graph", "execution", "stage"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.stage.unstaged_pass_count",
        1.0,
        &["graph", "execution", "stage"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.stage.unique_stage_count",
        7.0,
        &["graph", "execution", "stage"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.stage.transition_count",
        6.0,
        &["graph", "execution", "stage"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.stage.order_violation_count",
        0.0,
        &["graph", "execution", "stage", "order"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.profile.pass_count",
        3.0,
        &["graph", "execution", "profile"],
    );
    assert_render_microsecond_series(
        &snapshot.store,
        "render.graph.execution.profile.cpu_elapsed_total_us",
        425.0,
        &["graph", "execution", "profile", "cpu"],
    );
    assert_render_microsecond_series(
        &snapshot.store,
        "render.graph.execution.profile.cpu_elapsed_max_us",
        275.0,
        &["graph", "execution", "profile", "cpu"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_dispatch_count",
        2.0,
        &["graph", "compute", "dispatch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_dispatch_group_count",
        1234.0,
        &["graph", "compute", "dispatch"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_storage_write_resource_count",
        2.0,
        &["graph", "compute", "storage"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_planned_workload_count",
        2.0,
        &["graph", "compute", "workload"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_matched_workload_count",
        1.0,
        &["graph", "compute", "workload"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_missing_dispatch_count",
        1.0,
        &["graph", "compute", "workload"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_workload_mismatch_count",
        0.0,
        &["graph", "compute", "workload"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.compute_unexpected_dispatch_count",
        0.0,
        &["graph", "compute", "workload"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.debug_marker_count",
        14.0,
        &["graph", "debug_marker"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_anti_alias_pass_count",
        1.0,
        &["graph", "anti_alias"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_virtual_geometry_pass_count",
        2.0,
        &["graph", "virtual_geometry"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_hybrid_gi_pass_count",
        3.0,
        &["graph", "hybrid_gi"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_particle_pass_count",
        1.0,
        &["graph", "particle"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_shadow_pass_count",
        1.0,
        &["graph", "shadow"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_transparent_pass_count",
        4.0,
        &["graph", "transparent"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_async_compute_pass_count",
        2.0,
        &["graph", "async_compute"],
    );
}
