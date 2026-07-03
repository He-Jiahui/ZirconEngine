use super::*;

#[test]
fn runtime_15_ui_runtime_input_reply_routes_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/runtime_input_reply_routes.rs");
    let focus_text_accessibility =
        read_runtime_src("ui/tests/runtime_input_reply_routes/focus_text_accessibility_routes.rs");
    let pointer_bubble =
        read_runtime_src("ui/tests/runtime_input_reply_routes/pointer_bubble_routes.rs");
    let route_trace = read_runtime_src("ui/tests/runtime_input_reply_routes/route_trace_routes.rs");

    assert_contains_all(
        "UI runtime input reply routes parent mounts folder-backed children",
        &parent,
        &[
            "mod focus_text_accessibility_routes;",
            "mod pointer_bubble_routes;",
            "mod route_trace_routes;",
            "fn assert_two_node_bubble_handled_at_target(",
            "fn route_surface()",
            "fn press_release_route_surface()",
            "fn input_metadata()",
            "fn accessibility_event(",
        ],
    );
    assert_eq!(
        parent.matches(TEST_ATTRIBUTE).count(),
        0,
        "ui/tests/runtime_input_reply_routes.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "direct_dispatch_reply_populates_focus_route_trace_after_effects",
        "unified_pointer_press_release_report_bubble_route_steps_and_component_events",
        "unified_navigation_dispatch_reports_route_steps_and_focused_input_log",
        "accessibility_activate_dispatch_reports_owner_default_action_route_steps",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI runtime input reply route test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI runtime input reply route-trace child owns direct reply traces",
        &route_trace,
        &[
            "fn direct_dispatch_reply_populates_focus_route_trace_after_effects",
            "fn raw_mouse_motion_is_unrouted_by_surface_hit_testing",
            "fn dispatch_reply_steps_report_stopped_preview_and_focus_trace",
        ],
    );
    assert_contains_all(
        "UI runtime input reply pointer-bubble child owns unified pointer routes",
        &pointer_bubble,
        &[
            "fn unified_pointer_dispatch_reports_phase_route_steps",
            "fn pointer_preview_tunnel_handler_stops_before_target_and_bubble_handlers",
            "fn unified_pointer_double_click_reports_bubble_route_steps_and_default_binding",
            "fn unified_pointer_scroll_reports_bubble_route_steps_and_precise_delta",
        ],
    );
    assert_contains_all(
        "UI runtime input reply focus/text/accessibility child owns focus route surfaces",
        &focus_text_accessibility,
        &[
            "fn unified_focus_and_capture_dispatch_report_phase_route_steps",
            "fn unified_navigation_dispatch_reports_route_steps_and_focused_input_log",
            "fn unified_text_and_ime_dispatch_report_focus_route_steps_and_focused_input_log",
            "fn accessibility_activate_dispatch_reports_owner_default_action_route_steps",
        ],
    );

    let child_test_total = [
        focus_text_accessibility.as_str(),
        pointer_bubble.as_str(),
        route_trace.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches(TEST_ATTRIBUTE).count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 13,
        "UI runtime input reply route children should preserve all 13 parent tests"
    );

    for (path, source) in [
        ("ui/tests/runtime_input_reply_routes.rs", parent.as_str()),
        (
            "ui/tests/runtime_input_reply_routes/focus_text_accessibility_routes.rs",
            focus_text_accessibility.as_str(),
        ),
        (
            "ui/tests/runtime_input_reply_routes/pointer_bubble_routes.rs",
            pointer_bubble.as_str(),
        ),
        (
            "ui/tests/runtime_input_reply_routes/route_trace_routes.rs",
            route_trace.as_str(),
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
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/ui_tests_first.rs",
    );
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("UI architecture doc", ui_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 UI runtime input reply routes test folder split",
                "runtime_15_ui_runtime_input_reply_routes_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/runtime_input_reply_routes.rs",
                "ui/tests/runtime_input_reply_routes/route_trace_routes.rs",
                "ui/tests/runtime_input_reply_routes/focus_text_accessibility_routes.rs",
                "runtime_15_ui_runtime_input_reply_routes_tests_are_folder_backed",
            ],
        );
    }
}
