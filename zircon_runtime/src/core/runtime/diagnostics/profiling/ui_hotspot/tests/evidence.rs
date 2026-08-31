use super::*;

#[test]
fn ui_hotspots_preserve_hit_index_and_gpu_image_cache_evidence() {
    let mut snapshot = ProfileSnapshot::default();
    for (name, value) in [
        ("ui.idle_hover.workbench_hit_index_query_count", 8.0),
        ("ui.idle_hover.pane_popup_index_query_count", 4.0),
        ("ui.idle_hover.pane_popup_index_candidate_count", 0.0),
        ("ui.idle_hover.gpu_image_upload_writes", 1.0),
        ("ui.idle_hover.gpu_image_shared_resolves", 3.0),
        ("ui.idle_hover.gpu_image_prepare_cache_hits", 7.0),
        ("ui.idle_hover.gpu_image_cache_resident_bytes", 1024.0),
        ("ui.idle_hover.gpu_compiled_draw_items", 12.0),
        ("ui.idle_hover.gpu_batch_plan_builds", 3.0),
        ("ui.idle_hover.gpu_batch_plan_cache_hits", 0.0),
        ("ui.idle_hover.gpu_vertex_buffer_creates", 2.0),
        ("ui.idle_hover.gpu_vertex_upload_bytes", 4096.0),
        ("ui.idle_hover.gpu_retained_cache_copy_bytes", 8192.0),
        ("ui.idle_hover.visual_asset_cache_hit_count", 9.0),
        ("ui.idle_hover.svg_tree_cache_memory_hit_count", 2.0),
        (
            "ui.idle_hover.visual_asset_reconcile_source_visit_count",
            5.0,
        ),
        (
            "ui.idle_hover.visual_asset_reconciled_invalidation_count",
            1.0,
        ),
        ("ui.idle_hover.svg_tree_reconcile_source_visit_count", 3.0),
        ("ui.idle_hover.svg_tree_reconciled_invalidation_count", 1.0),
    ] {
        snapshot.counters.push(counter(name, value));
    }

    let report = super::analyze_ui_hotspots(&snapshot);
    let idle = report
        .scenarios
        .iter()
        .find(|scenario| scenario.scenario == "idle_hover")
        .expect("idle hover scenario");

    assert_eq!(idle.workbench_hit_index_query_count, 8);
    assert_eq!(idle.pane_popup_index_query_count, 4);
    assert_eq!(idle.pane_popup_index_candidate_count, 0);
    assert_eq!(idle.gpu_image_upload_write_count, 1);
    assert_eq!(idle.gpu_image_shared_resolve_count, 3);
    assert_eq!(idle.gpu_image_prepare_cache_hit_count, 7);
    assert_eq!(idle.gpu_image_cache_resident_bytes, 1024);
    assert_eq!(idle.gpu_compiled_draw_items, 12);
    assert_eq!(idle.gpu_batch_plan_build_count, 3);
    assert_eq!(idle.gpu_batch_plan_cache_hit_count, 0);
    assert_eq!(idle.gpu_vertex_buffer_create_count, 2);
    assert_eq!(idle.gpu_vertex_upload_bytes, 4096);
    assert_eq!(idle.gpu_retained_cache_copy_bytes, 8192);
    assert_eq!(idle.visual_asset_cache_hit_count, 9);
    assert_eq!(idle.svg_tree_cache_memory_hit_count, 2);
    assert_eq!(idle.visual_asset_reconcile_source_visit_count, 5);
    assert_eq!(idle.visual_asset_reconciled_invalidation_count, 1);
    assert_eq!(idle.svg_tree_reconcile_source_visit_count, 3);
    assert_eq!(idle.svg_tree_reconciled_invalidation_count, 1);
}
