use super::*;

#[test]
fn runtime_15_picking_tests_are_folder_backed() {
    let parent = read_runtime_src("tests/picking/mod.rs");
    let rays = read_runtime_src("tests/picking/rays.rs");
    let hits_and_hover = read_runtime_src("tests/picking/hits_and_hover.rs");
    let diagnostics = read_runtime_src("tests/picking/diagnostics.rs");
    let pipeline = read_runtime_src("tests/picking/pipeline.rs");
    let pointer_events = read_runtime_src("tests/picking/pointer_events.rs");

    assert_contains_all(
        "picking test parent mounts folder-backed children and shared fixtures",
        &parent,
        &[
            "mod diagnostics;",
            "mod hits_and_hover;",
            "mod pipeline;",
            "mod pointer_events;",
            "mod rays;",
            "fn hit(",
            "fn pointer_location(",
            "fn event_labels(",
            "fn test_camera(",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "tests/picking/mod.rs should only mount child test owners and shared fixtures"
    );
    for moved_test in [
        "perspective_pointer_location_builds_center_camera_ray",
        "hit_sorting_keeps_handle_gizmo_renderable_priority_before_depth",
        "picking_pipeline_report_counts_rays_outputs_and_hover_reduction",
        "picking_pipeline_runs_stages_and_carries_report",
        "pointer_event_state_drag_drop_and_scroll_sequence",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved picking test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "rays child owns viewport ray-map coverage",
        &rays,
        &[
            "fn perspective_pointer_location_builds_center_camera_ray",
            "fn ray_map_respects_pointer_viewport_and_camera_activity",
            "fn ray_map_builds_rays_for_two_pointers_and_two_active_cameras",
            "fn ray_map_keeps_same_pointer_locations_scoped_by_viewport",
            "fn ray_generation_uses_actual_viewport_aspect_for_off_center_pointers",
        ],
    );
    assert_contains_all(
        "hits and hover child owns picking priority and hover reduction coverage",
        &hits_and_hover,
        &[
            "fn hit_sorting_keeps_handle_gizmo_renderable_priority_before_depth",
            "fn hit_sorting_keeps_target_priority_before_backend_order",
            "fn hover_resolution_honors_non_hoverable_and_blocking_semantics",
            "fn primitive_backend_merges_multiple_ray_hits_by_existing_hover_rules",
            "fn hover_map_builds_from_multiple_backend_outputs",
        ],
    );
    assert_contains_all(
        "diagnostics child owns report and debug-feed coverage",
        &diagnostics,
        &[
            "fn picking_pipeline_report_counts_rays_outputs_and_hover_reduction",
            "fn picking_pipeline_report_exposes_blocking_non_hoverable_targets",
            "fn picking_debug_feed_exposes_summary_metrics_and_ray_only_rows",
            "fn picking_debug_feed_lists_blocked_non_hoverable_pointers",
        ],
    );
    assert_contains_all(
        "pipeline child owns runtime picking stage runner coverage",
        &pipeline,
        &[
            "fn picking_pipeline_runs_stages_and_carries_report",
            "fn disabled_picking_pipeline_clears_previous_interaction_state",
        ],
    );
    assert_contains_all(
        "pointer events child owns event-state sequence coverage",
        &pointer_events,
        &[
            "fn pointer_event_state_emits_hover_transitions_before_move",
            "fn pointer_event_state_click_release_use_previous_hover",
            "fn pointer_event_state_drag_drop_and_scroll_sequence",
            "fn pointer_event_state_cancel_filters_current_hover_and_clears_state",
        ],
    );

    for (path, source) in [
        ("tests/picking/mod.rs", parent.as_str()),
        ("tests/picking/rays.rs", rays.as_str()),
        ("tests/picking/hits_and_hover.rs", hits_and_hover.as_str()),
        ("tests/picking/diagnostics.rs", diagnostics.as_str()),
        ("tests/picking/pipeline.rs", pipeline.as_str()),
        ("tests/picking/pointer_events.rs", pointer_events.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let picking_doc = read_repo("docs/zircon_runtime/core/framework/picking.md");
}
