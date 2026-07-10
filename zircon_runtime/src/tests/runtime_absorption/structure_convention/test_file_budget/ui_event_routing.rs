use super::*;

#[test]
fn runtime_15_ui_event_routing_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/event_routing.rs");
    let component_events = read_runtime_src("ui/tests/event_routing/component_events.rs");
    let dispatch_effects = read_runtime_src("ui/tests/event_routing/dispatch_effects.rs");
    let pointer_state = read_runtime_src("ui/tests/event_routing/pointer_state.rs");
    let shared_input = read_runtime_src("ui/tests/event_routing/shared_input.rs");

    assert_contains_all(
        "UI event routing parent mounts folder-backed children",
        &parent,
        &[
            "mod component_events;",
            "mod dispatch_effects;",
            "mod pointer_state;",
            "mod shared_input;",
            "fn button_surface()",
            "fn bound_button_surface(",
            "fn two_button_surface(",
            "fn input_metadata()",
            "fn assert_render_only_dirty(",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/event_routing.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "primary_release_inside_pressed_target_marks_click_target_and_clears_press_state",
        "click_component_events_preserve_every_matching_binding_on_target",
        "dispatch_reply_applies_focus_capture_high_precision_and_release_effects",
        "shared_input_dispatch_routes_keyboard_text_ime_and_preserves_scroll_diagnostics",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI event-routing test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI event routing pointer-state child owns pointer routing and dirty-state tests",
        &pointer_state,
        &[
            "fn primary_release_inside_pressed_target_marks_click_target_and_clears_press_state",
            "fn pointer_dispatch_uses_virtual_pointer_query_for_component_events",
            "fn repeated_same_target_mouse_moves_do_not_dirty_or_rebuild_surface",
            "fn pointer_dispatch_reduces_hover_focus_and_press_into_component_state_store",
        ],
    );
    assert_contains_all(
        "UI event routing component-events child owns binding and scroll defaults",
        &component_events,
        &[
            "fn click_component_events_preserve_every_matching_binding_on_target",
            "fn focus_component_events_emit_focus_and_blur_for_matching_bindings",
            "fn scroll_fallback_continues_to_ancestor_when_nearest_scrollable_is_clamped",
        ],
    );
    assert_contains_all(
        "UI event routing dispatch-effects child owns host and focus effects",
        &dispatch_effects,
        &[
            "fn dispatch_reply_applies_focus_capture_high_precision_and_release_effects",
            "fn bound_custom_template_component_dispatches_click_envelope_after_build",
            "fn focus_effects_clear_only_their_current_input_owner",
            "fn dispatch_reply_applies_navigation_and_host_owned_input_effects",
            "fn input_method_request_rejects_invalid_surrounding_text_before_host_request",
        ],
    );
    assert_contains_all(
        "UI event routing shared-input child owns keyboard, text, IME, and rejection tests",
        &shared_input,
        &[
            "fn shared_input_dispatch_routes_keyboard_text_ime_and_preserves_scroll_diagnostics",
            "fn shared_text_input_mutates_focused_editable_value_and_marks_text_dirty",
            "fn shared_ime_preedit_commit_and_cancel_mutate_editable_composition",
            "fn shared_input_dispatch_rejects_invalid_owners_and_hidden_ancestors",
        ],
    );

    let child_test_total = [
        component_events.as_str(),
        dispatch_effects.as_str(),
        pointer_state.as_str(),
        shared_input.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 27,
        "UI event-routing children should preserve all 27 parent tests"
    );

    for (path, source) in [
        ("ui/tests/event_routing.rs", parent.as_str()),
        (
            "ui/tests/event_routing/component_events.rs",
            component_events.as_str(),
        ),
        (
            "ui/tests/event_routing/dispatch_effects.rs",
            dispatch_effects.as_str(),
        ),
        (
            "ui/tests/event_routing/pointer_state.rs",
            pointer_state.as_str(),
        ),
        (
            "ui/tests/event_routing/shared_input.rs",
            shared_input.as_str(),
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
    let status_rows = ui_tests_first_status_row_source();
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
                "Runtime 15 M3 UI event routing test folder split",
                "runtime_15_ui_event_routing_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/event_routing.rs",
                "ui/tests/event_routing/pointer_state.rs",
                "ui/tests/event_routing/shared_input.rs",
                "runtime_15_ui_event_routing_tests_are_folder_backed",
            ],
        );
    }
}
