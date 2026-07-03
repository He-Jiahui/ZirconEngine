use super::*;

#[test]
fn runtime_15_ui_runtime_input_manager_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/runtime_input_manager.rs");
    let route_matrix = read_runtime_src("ui/tests/runtime_input_manager/route_matrix.rs");
    let route_order = read_runtime_src("ui/tests/runtime_input_manager/route_order.rs");
    let touch_pointer = read_runtime_src("ui/tests/runtime_input_manager/touch_pointer.rs");
    let window_timer = read_runtime_src("ui/tests/runtime_input_manager/window_timer.rs");

    assert_contains_all(
        "UI runtime input manager parent mounts folder-backed children and keeps helpers",
        &parent,
        &[
            "mod route_matrix;",
            "mod route_order;",
            "mod touch_pointer;",
            "mod window_timer;",
            "fn route_matrix_surface()",
            "fn double_click_manager_surface()",
            "fn popup_matrix_surface()",
            "fn pointer_event_at(",
            "fn touch_pointer_event_at(",
            "fn window_metadata(",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/runtime_input_manager.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "input_manager_window_batch_aggregates_results_and_redraw_requests",
        "input_manager_route_order_matches_slate_style_authority_order",
        "input_manager_route_matrix_capture_preempts_hit_target",
        "input_manager_double_click_count_is_owned_by_timer_state",
        "input_manager_multi_pointer_capture_isolation_survives_cancel",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI runtime input manager test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI runtime input manager window-timer child owns pump and tick tests",
        &window_timer,
        &[
            "fn input_manager_window_batch_aggregates_results_and_redraw_requests",
            "fn input_manager_tick_records_timer_owner_state",
        ],
    );
    assert_contains_all(
        "UI runtime input manager route-order child owns authority order tests",
        &route_order,
        &[
            "fn input_manager_route_order_matches_slate_style_authority_order",
            "fn input_manager_route_policy_stage_names_follow_authority_order",
        ],
    );
    assert_contains_all(
        "UI runtime input manager route-matrix child owns capture, popup, preview, focus, and default-action routes",
        &route_matrix,
        &[
            "fn input_manager_route_matrix_capture_preempts_hit_target",
            "fn input_manager_route_matrix_popup_outside_closes_top_only",
            "fn input_manager_route_matrix_preview_stops_before_bubble",
            "fn input_manager_route_matrix_keyboard_uses_focus_path",
            "fn input_manager_route_matrix_popup_open_uses_default_action",
        ],
    );
    assert_contains_all(
        "UI runtime input manager touch-pointer child owns double-click and multi-pointer contracts",
        &touch_pointer,
        &[
            "fn input_manager_double_click_count_is_owned_by_timer_state",
            "fn input_manager_primary_touch_synthesizes_mouse_click",
            "fn input_manager_secondary_touch_keeps_table_press_without_mouse_activation",
            "fn input_manager_touch_cancel_clears_pointer_entry_and_capture",
            "fn input_manager_two_touch_pointers_keep_independent_hover_and_press",
            "fn input_manager_multi_pointer_capture_isolation_survives_cancel",
        ],
    );

    let child_test_total = [
        route_matrix.as_str(),
        route_order.as_str(),
        touch_pointer.as_str(),
        window_timer.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 15,
        "UI runtime input manager children should preserve all 15 parent tests"
    );

    for (path, source) in [
        ("ui/tests/runtime_input_manager.rs", parent.as_str()),
        (
            "ui/tests/runtime_input_manager/route_matrix.rs",
            route_matrix.as_str(),
        ),
        (
            "ui/tests/runtime_input_manager/route_order.rs",
            route_order.as_str(),
        ),
        (
            "ui/tests/runtime_input_manager/touch_pointer.rs",
            touch_pointer.as_str(),
        ),
        (
            "ui/tests/runtime_input_manager/window_timer.rs",
            window_timer.as_str(),
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
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/ui_tests_second.rs",
    );
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
                "Runtime 15 M3 UI runtime input manager test folder split",
                "runtime_15_ui_runtime_input_manager_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/runtime_input_manager.rs",
                "ui/tests/runtime_input_manager/route_matrix.rs",
                "ui/tests/runtime_input_manager/touch_pointer.rs",
                "runtime_15_ui_runtime_input_manager_tests_are_folder_backed",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M3 UI runtime input manager test folder split",
            "runtime_15_ui_runtime_input_manager_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/runtime_input_manager.rs",
            "ui/tests/runtime_input_manager/route_matrix.rs",
            "runtime_15_ui_runtime_input_manager_tests_are_folder_backed",
        ],
    );
}
