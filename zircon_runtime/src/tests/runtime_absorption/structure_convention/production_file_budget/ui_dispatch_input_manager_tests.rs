use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_ui_dispatch_input_manager_tests_are_child_owner() {
    let parent = read_runtime_src("ui/dispatch/input_manager/manager.rs");
    let tests = read_runtime_src("ui/dispatch/input_manager/manager/tests.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );

    assert_contains_all(
        "UI input manager parent keeps production dispatch API and child test mount",
        &parent,
        &[
            "pub struct UiInputManager",
            "pub fn dispatch_input_event(",
            "pub fn dispatch_window_input_pump_event(",
            "pub fn dispatch_window_input_pump_batch(",
            "pub fn tick(",
            "#[cfg(test)]\nmod tests;",
        ],
    );
    for moved_test in [
        "fn hovered_menu_option_arms_replaces_and_clears_submenu_hover_timer(",
        "fn popup_menu_shells_expose_typeahead_and_submenu_timer_contracts(",
        "fn toast_queue_value_arms_replaces_and_clears_auto_hide_timer(",
        "fn toast_auto_hide_tick_dispatches_expired_commit_event(",
        "fn tooltip_hover_arms_and_clears_manager_timer_candidate(",
        "fn tooltip_hover_timer_tick_dispatches_elapsed_default_action(",
        "fn tooltip_candidate_clears_on_following_input_activity(",
    ] {
        assert!(
            !parent.contains(moved_test),
            "ui/dispatch/input_manager/manager.rs should delegate test owner `{moved_test}` to manager/tests.rs"
        );
        assert!(
            tests.contains(moved_test),
            "ui/dispatch/input_manager/manager/tests.rs should own moved test `{moved_test}`"
        );
    }

    assert_contains_all(
        "UI input manager test child owns timer coverage and test-only fixtures",
        &tests,
        &[
            "use super::UiInputManager;",
            "fn submenu_hover_surface(",
            "fn toast_surface(",
            "fn tooltip_surface(",
            "fn hover_changed_result(",
            "fn component_event_result(",
            "fn binding(",
            "UiTooltipTimerInputEventKind",
            "UiDispatchHostRequestKind",
        ],
    );

    for (path, source) in [
        ("ui/dispatch/input_manager/manager.rs", parent.as_str()),
        ("ui/dispatch/input_manager/manager/tests.rs", tests.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

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
                "Runtime 15 M4 UI dispatch input manager test owner split",
                "runtime_15_ui_dispatch_input_manager_tests_owner_split_static_passed_cargo_deferred",
                "ui/dispatch/input_manager/manager.rs",
                "ui/dispatch/input_manager/manager/tests.rs",
                "runtime_15_ui_dispatch_input_manager_tests_are_child_owner",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M4 UI dispatch input manager test owner split",
            "runtime_15_ui_dispatch_input_manager_tests_owner_split_static_passed_cargo_deferred",
            "ui/dispatch/input_manager/manager.rs",
            "ui/dispatch/input_manager/manager/tests.rs",
            "runtime_15_ui_dispatch_input_manager_tests_are_child_owner",
        ],
    );
}
