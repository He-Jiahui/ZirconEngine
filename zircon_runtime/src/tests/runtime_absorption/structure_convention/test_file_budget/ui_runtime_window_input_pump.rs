use super::*;

#[test]
fn runtime_15_ui_runtime_window_input_pump_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/runtime_window_input_pump.rs");
    let lifecycle = read_runtime_src("ui/tests/runtime_window_input_pump/lifecycle.rs");
    let metrics_dirty = read_runtime_src("ui/tests/runtime_window_input_pump/metrics_dirty.rs");
    let pointer_routes = read_runtime_src("ui/tests/runtime_window_input_pump/pointer_routes.rs");

    assert_contains_all(
        "UI runtime window input pump parent mounts folder-backed children",
        &parent,
        &[
            "mod lifecycle;",
            "mod metrics_dirty;",
            "mod pointer_routes;",
            "fn route_surface(",
            "fn dispatch_window_input_pump_event(",
            "fn window_metadata(",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/runtime_window_input_pump.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "window_input_pump_app_deactivation_closes_popup_stack_and_tooltip",
        "window_input_pump_cursor_move_dispatches_unified_pointer_hover_route",
        "window_input_pump_resize_updates_frame_metrics_and_layout_dirty_domains",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI runtime window input pump test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI runtime window input pump lifecycle child owns lifecycle contracts",
        &lifecycle,
        &[
            "fn window_input_pump_app_deactivation_closes_popup_stack_and_tooltip",
            "fn window_input_pump_focus_loss_closes_popup_stack_and_tooltip",
            "fn window_input_pump_retains_focus_activation_and_occlusion_facts",
            "fn window_input_pump_batch_preserves_order_and_non_client_area_keeps_tooltip",
            "fn window_input_pump_retains_close_request_without_closing_the_surface",
            "fn window_input_pump_destroyed_retains_lifecycle_fact_and_clears_hover",
        ],
    );
    assert_contains_all(
        "UI runtime window input pump pointer child owns pointer routes",
        &pointer_routes,
        &[
            "fn window_input_pump_cursor_move_dispatches_unified_pointer_hover_route",
            "fn window_input_pump_cursor_left_replays_pointer_cancel_and_clears_hover",
            "fn window_input_pump_touch_move_does_not_replace_last_mouse_cursor_point",
            "fn window_input_pump_closed_without_cursor_point_clears_hover_without_fake_pointer_cancel",
        ],
    );
    assert_contains_all(
        "UI runtime window input pump metrics child owns metrics and dirty domains",
        &metrics_dirty,
        &[
            "fn window_input_pump_resize_updates_frame_metrics_and_layout_dirty_domains",
            "fn window_input_pump_scale_factor_updates_retained_metrics_without_losing_size",
            "fn window_input_pump_move_updates_position_without_dirty_domains",
            "fn window_input_pump_redraw_request_marks_render_dirty_only",
        ],
    );

    let child_test_total = [
        lifecycle.as_str(),
        metrics_dirty.as_str(),
        pointer_routes.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 14,
        "UI runtime window input pump children should preserve all 14 parent tests"
    );

    for (path, source) in [
        ("ui/tests/runtime_window_input_pump.rs", parent.as_str()),
        (
            "ui/tests/runtime_window_input_pump/lifecycle.rs",
            lifecycle.as_str(),
        ),
        (
            "ui/tests/runtime_window_input_pump/metrics_dirty.rs",
            metrics_dirty.as_str(),
        ),
        (
            "ui/tests/runtime_window_input_pump/pointer_routes.rs",
            pointer_routes.as_str(),
        ),
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
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");
    let status_rows = ui_tests_second_status_row_source();
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("UI architecture doc", ui_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 UI runtime window input pump test folder split",
                "runtime_15_ui_runtime_window_input_pump_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/runtime_window_input_pump.rs",
                "ui/tests/runtime_window_input_pump/lifecycle.rs",
                "ui/tests/runtime_window_input_pump/pointer_routes.rs",
                "runtime_15_ui_runtime_window_input_pump_tests_are_folder_backed",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M3 UI runtime window input pump test folder split",
            "runtime_15_ui_runtime_window_input_pump_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/runtime_window_input_pump.rs",
            "ui/tests/runtime_window_input_pump/lifecycle.rs",
            "runtime_15_ui_runtime_window_input_pump_tests_are_folder_backed",
        ],
    );
}
