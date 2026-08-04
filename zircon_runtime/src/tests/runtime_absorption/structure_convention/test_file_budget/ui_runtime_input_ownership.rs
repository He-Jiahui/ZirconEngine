use super::*;

#[test]
fn runtime_15_ui_runtime_input_ownership_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/runtime_input_ownership.rs");
    let drag_drop = read_runtime_src("ui/tests/runtime_input_ownership/drag_drop.rs");
    let high_precision_dispatch =
        read_runtime_src("ui/tests/runtime_input_ownership/high_precision_dispatch.rs");
    let input_method = read_runtime_src("ui/tests/runtime_input_ownership/input_method.rs");
    let owner_validation = read_runtime_src("ui/tests/runtime_input_ownership/owner_validation.rs");
    let popup_tooltip = read_runtime_src("ui/tests/runtime_input_ownership/popup_tooltip.rs");
    let route_trace = read_runtime_src("ui/tests/runtime_input_ownership/route_trace.rs");

    assert_contains_all(
        "UI runtime input ownership parent mounts folder-backed children and keeps helpers",
        &parent,
        &[
            "mod drag_drop;",
            "mod high_precision_dispatch;",
            "mod input_method;",
            "mod owner_validation;",
            "mod popup_tooltip;",
            "mod route_trace;",
            "fn capture_pointer_for_test(",
            "fn assert_pointer_capture(",
            "fn assert_no_pointer_capture(",
            "fn two_button_surface(",
            "fn input_metadata(",
            "fn drag_effect(",
            "fn input_method_request(",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/runtime_input_ownership.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "rejected_focus_effect_preserves_current_input_method_owner",
        "focus_and_capture_reject_hidden_ancestor_owners_without_clearing_current_owner",
        "high_precision_requires_capture_and_release_clears_only_matching_owner",
        "drag_drop_lifecycle_tracks_shared_state_and_cleans_capture_on_end",
        "popup_and_tooltip_inputs_reject_stale_owner_without_mutating_shared_state",
        "unified_input_dispatch_trace_reports_capture_and_popup_stack",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI runtime input ownership test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI runtime input ownership input-method child owns IME ownership contracts",
        &input_method,
        &[
            "fn rejected_focus_effect_preserves_current_input_method_owner",
            "fn navigation_focus_changes_clear_previous_input_method_owner",
            "fn clear_focus_clears_only_the_focused_input_method_owner",
            "fn input_method_reset_and_cursor_update_require_current_owner",
        ],
    );
    assert_contains_all(
        "UI runtime input ownership validation child owns hidden/disabled owner contracts",
        &owner_validation,
        &[
            "fn focus_and_capture_reject_hidden_ancestor_owners_without_clearing_current_owner",
            "fn mutating_disabled_ancestor_clears_focus_and_transient_input_owners",
            "fn direct_capture_without_pointer_id_does_not_enable_high_precision",
        ],
    );
    assert_contains_all(
        "UI runtime input ownership high-precision child owns capture dispatch contracts",
        &high_precision_dispatch,
        &[
            "fn high_precision_requires_capture_and_release_clears_only_matching_owner",
            "fn reply_step_route_stops_before_later_bubble_effects",
        ],
    );
    assert_contains_all(
        "UI runtime input ownership drag-drop child owns drag lifecycle contracts",
        &drag_drop,
        &[
            "fn drag_drop_lifecycle_tracks_shared_state_and_cleans_capture_on_end",
            "fn drag_drop_rejects_stale_pointer_or_session_without_clearing_active_drag",
        ],
    );
    assert_contains_all(
        "UI runtime input ownership popup-tooltip child owns transient UI input contracts",
        &popup_tooltip,
        &[
            "fn popup_and_tooltip_inputs_reject_stale_owner_without_mutating_shared_state",
            "fn shared_input_dispatch_applies_drag_drop_popup_and_tooltip_events_through_effects",
        ],
    );
    assert_contains_all(
        "UI runtime input ownership route-trace child owns shared input route contracts",
        &route_trace,
        &[
            "fn analog_input_suppresses_repeated_values_before_routing",
            "fn unified_input_dispatch_reports_slate_style_pointer_and_focus_route_trace",
            "fn unified_input_dispatch_trace_reports_capture_and_popup_stack",
        ],
    );

    let child_test_total = [
        drag_drop.as_str(),
        high_precision_dispatch.as_str(),
        input_method.as_str(),
        owner_validation.as_str(),
        popup_tooltip.as_str(),
        route_trace.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 16,
        "UI runtime input ownership children should preserve all 16 parent tests"
    );

    for (path, source) in [
        ("ui/tests/runtime_input_ownership.rs", parent.as_str()),
        (
            "ui/tests/runtime_input_ownership/drag_drop.rs",
            drag_drop.as_str(),
        ),
        (
            "ui/tests/runtime_input_ownership/high_precision_dispatch.rs",
            high_precision_dispatch.as_str(),
        ),
        (
            "ui/tests/runtime_input_ownership/input_method.rs",
            input_method.as_str(),
        ),
        (
            "ui/tests/runtime_input_ownership/owner_validation.rs",
            owner_validation.as_str(),
        ),
        (
            "ui/tests/runtime_input_ownership/popup_tooltip.rs",
            popup_tooltip.as_str(),
        ),
        (
            "ui/tests/runtime_input_ownership/route_trace.rs",
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
                "Runtime 15 M3 UI runtime input ownership test folder split",
                "runtime_15_ui_runtime_input_ownership_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/runtime_input_ownership.rs",
                "ui/tests/runtime_input_ownership/input_method.rs",
                "ui/tests/runtime_input_ownership/route_trace.rs",
                "runtime_15_ui_runtime_input_ownership_tests_are_folder_backed",
            ],
        );
    }
}
