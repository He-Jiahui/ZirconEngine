use super::*;

#[test]
fn runtime_15_ui_focus_navigation_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/focus_navigation.rs");
    let focus_state = read_runtime_src("ui/tests/focus_navigation/focus_state.rs");
    let modal_popup = read_runtime_src("ui/tests/focus_navigation/modal_popup.rs");
    let property_mutation = read_runtime_src("ui/tests/focus_navigation/property_mutation.rs");
    let tab_directional = read_runtime_src("ui/tests/focus_navigation/tab_directional.rs");

    assert_contains_all(
        "UI focus navigation parent mounts folder-backed children and keeps helpers",
        &parent,
        &[
            "mod focus_state;",
            "mod modal_popup;",
            "mod property_mutation;",
            "mod tab_directional;",
            "fn focus_surface(",
            "fn mui_modal_component_surface(",
            "fn popup_focus_surface(",
            "fn navigation_surface(",
            "fn input_metadata(",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/focus_navigation.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "autofocus_records_initial_focus_change_and_visible_reason",
        "focus_is_cleared_when_focused_node_stops_accepting_input",
        "tab_navigation_uses_index_order_and_modal_group_trap",
        "widget_popup_open_traps_focus_loop_and_restores_previous_focus",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI focus navigation test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI focus navigation state child owns focus input-state contracts",
        &focus_state,
        &[
            "fn autofocus_records_initial_focus_change_and_visible_reason",
            "fn pointer_and_navigation_focus_sources_update_visible_reason",
            "fn focus_component_state_changes_mark_render_only_dirty",
            "fn text_and_ime_inputs_record_focused_input_routes",
        ],
    );
    assert_contains_all(
        "UI focus navigation mutation child owns property focus contracts",
        &property_mutation,
        &[
            "fn authored_focus_contract_makes_node_focusable_without_legacy_state_flag",
            "fn focus_is_cleared_when_focused_node_stops_accepting_input",
            "fn focus_is_cleared_when_focused_node_ancestor_is_disabled",
            "fn unchanged_or_rejected_focus_related_mutations_do_not_emit_focus_changes",
        ],
    );
    assert_contains_all(
        "UI focus navigation tab/directional child owns navigation contracts",
        &tab_directional,
        &[
            "fn tab_navigation_uses_index_order_and_modal_group_trap",
            "fn tab_navigation_crosses_non_modal_groups_by_group_order",
            "fn directional_navigation_honors_manual_overrides_and_blocked_edges",
            "fn modal_directional_navigation_rejects_manual_targets_outside_modal_group",
        ],
    );
    assert_contains_all(
        "UI focus navigation modal/popup child owns focus trap contracts",
        &modal_popup,
        &[
            "fn mui_modal_open_autofocus_traps_navigation_and_restores_previous_focus",
            "fn confirm_dialog_popup_open_autofocus_traps_navigation_and_restores_previous_focus",
            "fn mui_modal_focus_flags_can_disable_auto_enforce_and_restore",
            "fn widget_popup_open_traps_focus_loop_and_restores_previous_focus",
        ],
    );

    let child_test_total = [
        focus_state.as_str(),
        modal_popup.as_str(),
        property_mutation.as_str(),
        tab_directional.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 18,
        "UI focus navigation children should preserve all 18 current tests"
    );

    for (path, source) in [
        ("ui/tests/focus_navigation.rs", parent.as_str()),
        (
            "ui/tests/focus_navigation/focus_state.rs",
            focus_state.as_str(),
        ),
        (
            "ui/tests/focus_navigation/modal_popup.rs",
            modal_popup.as_str(),
        ),
        (
            "ui/tests/focus_navigation/property_mutation.rs",
            property_mutation.as_str(),
        ),
        (
            "ui/tests/focus_navigation/tab_directional.rs",
            tab_directional.as_str(),
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
                "Runtime 15 M3 UI focus navigation test folder split",
                "runtime_15_ui_focus_navigation_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/focus_navigation.rs",
                "ui/tests/focus_navigation/focus_state.rs",
                "ui/tests/focus_navigation/modal_popup.rs",
                "runtime_15_ui_focus_navigation_tests_are_folder_backed",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M3 UI focus navigation test folder split",
            "runtime_15_ui_focus_navigation_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/focus_navigation.rs",
            "ui/tests/focus_navigation/focus_state.rs",
            "runtime_15_ui_focus_navigation_tests_are_folder_backed",
        ],
    );
}
