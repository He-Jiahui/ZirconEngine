use super::*;

#[test]
fn runtime_15_ui_runtime_window_event_abi_children_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/runtime_ui_window_event_routes/abi.rs");
    let batch_adapter =
        read_runtime_src("ui/tests/runtime_ui_window_event_routes/abi/batch_adapter.rs");
    let keyboard_gamepad =
        read_runtime_src("ui/tests/runtime_ui_window_event_routes/abi/keyboard_gamepad_routes.rs");
    let pointer_window =
        read_runtime_src("ui/tests/runtime_ui_window_event_routes/abi/pointer_window_routes.rs");

    assert_contains_all(
        "UI runtime window event ABI parent mounts folder-backed children",
        &parent,
        &[
            "use super::*;",
            "mod batch_adapter;",
            "mod keyboard_gamepad_routes;",
            "mod pointer_window_routes;",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/runtime_ui_window_event_routes/abi.rs should only mount child test owners"
    );
    for moved_test in [
        "runtime_ui_manager_dispatches_runtime_event_batch_through_window_adapter",
        "runtime_ui_manager_routes_runtime_pointer_events_through_owned_dispatcher",
        "runtime_ui_manager_routes_runtime_keyboard_enter_through_focused_window_path",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI runtime window event ABI test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI runtime window event ABI batch child owns batch and adapter contracts",
        &batch_adapter,
        &[
            "fn runtime_ui_manager_dispatches_runtime_event_batch_through_window_adapter",
            "fn runtime_ui_manager_runtime_event_batch_rebuilds_before_followup_pointer_input",
            "fn runtime_ui_manager_runtime_event_batch_keeps_prior_events_when_later_adapter_fails",
            "fn runtime_ui_manager_runtime_event_batch_reports_dispatch_error_index_after_adapter_success",
            "fn runtime_ui_manager_reports_runtime_event_adapter_errors_without_surface_mutation",
        ],
    );
    assert_contains_all(
        "UI runtime window event ABI pointer child owns pointer/window routes",
        &pointer_window,
        &[
            "fn runtime_ui_manager_routes_runtime_pointer_events_through_owned_dispatcher",
            "fn runtime_ui_manager_routes_runtime_wheel_at_point_through_owned_scroll_route",
            "fn runtime_ui_manager_routes_runtime_pointer_moved_through_window_hover_pump",
            "fn runtime_ui_manager_routes_runtime_cursor_left_through_window_pointer_cancel",
            "fn runtime_ui_manager_routes_runtime_touch_events_through_owned_dispatcher",
        ],
    );
    assert_contains_all(
        "UI runtime window event ABI keyboard child owns keyboard/gamepad routes",
        &keyboard_gamepad,
        &[
            "fn runtime_ui_manager_routes_runtime_keyboard_enter_through_focused_window_path",
            "fn runtime_ui_manager_routes_runtime_gamepad_dpad_right_through_focused_navigation_path",
            "fn runtime_ui_manager_routes_runtime_gamepad_axis_right_through_focused_analog_navigation_path",
        ],
    );

    let child_test_total = [
        batch_adapter.as_str(),
        keyboard_gamepad.as_str(),
        pointer_window.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 13,
        "UI runtime window event ABI children should preserve all 13 parent tests"
    );

    for (path, source) in [
        (
            "ui/tests/runtime_ui_window_event_routes/abi.rs",
            parent.as_str(),
        ),
        (
            "ui/tests/runtime_ui_window_event_routes/abi/batch_adapter.rs",
            batch_adapter.as_str(),
        ),
        (
            "ui/tests/runtime_ui_window_event_routes/abi/keyboard_gamepad_routes.rs",
            keyboard_gamepad.as_str(),
        ),
        (
            "ui/tests/runtime_ui_window_event_routes/abi/pointer_window_routes.rs",
            pointer_window.as_str(),
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
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
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
                "Runtime 15 M3 UI runtime window event ABI child folder split",
                "runtime_15_ui_runtime_window_event_abi_children_folder_split_static_passed_cargo_deferred",
                "ui/tests/runtime_ui_window_event_routes/abi.rs",
                "ui/tests/runtime_ui_window_event_routes/abi/batch_adapter.rs",
                "ui/tests/runtime_ui_window_event_routes/abi/pointer_window_routes.rs",
                "runtime_15_ui_runtime_window_event_abi_children_are_folder_backed",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M3 UI runtime window event ABI child folder split",
            "runtime_15_ui_runtime_window_event_abi_children_folder_split_static_passed_cargo_deferred",
            "ui/tests/runtime_ui_window_event_routes/abi.rs",
            "ui/tests/runtime_ui_window_event_routes/abi/batch_adapter.rs",
            "runtime_15_ui_runtime_window_event_abi_children_are_folder_backed",
        ],
    );
}
