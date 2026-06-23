use crate::core::diagnostics::RuntimeDiagnosticsSnapshot;

use super::support::{assert_render_byte_series, assert_render_count_series};

pub(super) fn assert_graph_resources(snapshot: &RuntimeDiagnosticsSnapshot) {
    assert_render_count_series(
        &snapshot.store,
        "render.last_graph_executed_pass_count",
        14.0,
        &["graph"],
    );
    assert_render_count_series(&snapshot.store, "render.graph.pass_count", 18.0, &["graph"]);
    assert_render_count_series(
        &snapshot.store,
        "render.graph.culled_pass_count",
        4.0,
        &["graph", "culling"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.queue_fallback_pass_count",
        2.0,
        &["graph", "queue"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.resource_lifetime_count",
        6.0,
        &["graph", "resource"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.sparse_texture_lifetime_count",
        1.0,
        &["graph", "resource", "sparse_texture"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.planned_resource_access_count",
        22.0,
        &["graph", "resource"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.planned_dependency_count",
        9.0,
        &["graph", "dependency"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.transient_texture_slot_count",
        3.0,
        &["graph", "transient", "texture"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.sparse_texture_slot_count",
        1.0,
        &["graph", "transient", "texture", "sparse_texture"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.transient_buffer_slot_count",
        2.0,
        &["graph", "transient", "buffer"],
    );
    assert_render_byte_series(
        &snapshot.store,
        "render.graph.transient_texture_bytes_reserved",
        4_194_304.0,
        &["graph", "transient", "texture"],
    );
    assert_render_byte_series(
        &snapshot.store,
        "render.graph.transient_buffer_bytes_reserved",
        65_536.0,
        &["graph", "transient", "buffer"],
    );
    assert_render_byte_series(
        &snapshot.store,
        "render.graph.transient_dense_bytes_reserved",
        4_259_840.0,
        &["graph", "transient"],
    );
    assert_render_byte_series(
        &snapshot.store,
        "render.graph.sparse_texture_virtual_bytes",
        16_777_216.0,
        &["graph", "transient", "texture", "sparse_texture"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_pass_count",
        14.0,
        &["graph"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_resource_access_count",
        19.0,
        &["graph", "resource"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.executed_dependency_count",
        8.0,
        &["graph", "dependency"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.texture_view_count",
        18.0,
        &["graph", "execution", "resource", "texture"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.external_texture_view_count",
        14.0,
        &["graph", "execution", "resource", "texture", "external"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.owned_texture_count",
        4.0,
        &["graph", "execution", "resource", "texture", "owned"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.buffer_count",
        3.0,
        &["graph", "execution", "resource", "buffer"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.bound_resource_count",
        21.0,
        &["graph", "execution", "resource"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.transient_pool.texture_created_count",
        5.0,
        &[
            "graph",
            "execution",
            "resource",
            "pool",
            "texture",
            "created",
        ],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.transient_pool.texture_reused_count",
        7.0,
        &[
            "graph",
            "execution",
            "resource",
            "pool",
            "texture",
            "reused",
        ],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.transient_pool.buffer_created_count",
        2.0,
        &[
            "graph",
            "execution",
            "resource",
            "pool",
            "buffer",
            "created",
        ],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.transient_pool.buffer_reused_count",
        3.0,
        &["graph", "execution", "resource", "pool", "buffer", "reused"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.transient_pool.texture_pool_entry_count",
        4.0,
        &["graph", "execution", "resource", "pool", "texture"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.transient_pool.buffer_pool_entry_count",
        1.0,
        &["graph", "execution", "resource", "pool", "buffer"],
    );
    assert_render_byte_series(
        &snapshot.store,
        "render.graph.execution.transient_pool.texture_pool_retained_bytes",
        4_096.0,
        &[
            "graph",
            "execution",
            "resource",
            "pool",
            "texture",
            "retained",
        ],
    );
    assert_render_byte_series(
        &snapshot.store,
        "render.graph.execution.transient_pool.buffer_pool_retained_bytes",
        512.0,
        &[
            "graph",
            "execution",
            "resource",
            "pool",
            "buffer",
            "retained",
        ],
    );
    assert_render_byte_series(
        &snapshot.store,
        "render.graph.execution.transient_pool.texture_pool_budget_bytes",
        1_048_576.0,
        &[
            "graph",
            "execution",
            "resource",
            "pool",
            "texture",
            "budget",
        ],
    );
    assert_render_byte_series(
        &snapshot.store,
        "render.graph.execution.transient_pool.buffer_pool_budget_bytes",
        65_536.0,
        &["graph", "execution", "resource", "pool", "buffer", "budget"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.transient_pool.evicted_texture_count",
        8.0,
        &[
            "graph",
            "execution",
            "resource",
            "pool",
            "texture",
            "evicted",
        ],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.transient_pool.evicted_buffer_count",
        9.0,
        &[
            "graph",
            "execution",
            "resource",
            "pool",
            "buffer",
            "evicted",
        ],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.transient_pool.budget_evicted_texture_count",
        10.0,
        &[
            "graph",
            "execution",
            "resource",
            "pool",
            "texture",
            "budget",
            "evicted",
        ],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.execution.transient_pool.budget_evicted_buffer_count",
        11.0,
        &[
            "graph",
            "execution",
            "resource",
            "pool",
            "buffer",
            "budget",
            "evicted",
        ],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.materialization.required_resource_count",
        9.0,
        &["graph", "materialization", "resource"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.materialization.bound_resource_count",
        11.0,
        &["graph", "materialization", "resource", "bound"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.materialization.missing_resource_count",
        1.0,
        &["graph", "materialization", "resource", "missing"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.materialization.missing_materialized_resource_count",
        0.0,
        &["graph", "materialization", "resource", "missing", "typed"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.materialization.required_texture_count",
        4.0,
        &["graph", "materialization", "texture"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.materialization.bound_texture_count",
        4.0,
        &["graph", "materialization", "texture", "bound"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.materialization.missing_texture_count",
        0.0,
        &["graph", "materialization", "texture", "missing"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.materialization.required_buffer_count",
        3.0,
        &["graph", "materialization", "buffer"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.materialization.bound_buffer_count",
        3.0,
        &["graph", "materialization", "buffer", "bound"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.materialization.missing_buffer_count",
        0.0,
        &["graph", "materialization", "buffer", "missing"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.materialization.required_external_count",
        2.0,
        &["graph", "materialization", "external"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.materialization.bound_external_count",
        4.0,
        &["graph", "materialization", "external", "bound"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.materialization.missing_external_count",
        1.0,
        &["graph", "materialization", "external", "missing"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.materialization.bound_required_external_count",
        2.0,
        &["graph", "materialization", "external", "required", "bound"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.materialization.missing_required_external_count",
        0.0,
        &[
            "graph",
            "materialization",
            "external",
            "required",
            "missing",
        ],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.materialization.report_only_external_count",
        3.0,
        &["graph", "materialization", "external", "report_only"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.materialization.bound_report_only_external_count",
        2.0,
        &[
            "graph",
            "materialization",
            "external",
            "report_only",
            "bound",
        ],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.materialization.missing_report_only_external_count",
        1.0,
        &[
            "graph",
            "materialization",
            "external",
            "report_only",
            "missing",
        ],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.materialization.stale_binding_count",
        0.0,
        &["graph", "materialization", "resource", "stale_binding"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.materialization.stale_texture_binding_count",
        0.0,
        &["graph", "materialization", "texture", "stale_binding"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.materialization.stale_buffer_binding_count",
        0.0,
        &["graph", "materialization", "buffer", "stale_binding"],
    );
    assert_render_count_series(
        &snapshot.store,
        "render.graph.materialization.sparse_texture_reservation_count",
        1.0,
        &["graph", "materialization", "texture", "sparse_texture"],
    );
}
