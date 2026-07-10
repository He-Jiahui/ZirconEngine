use super::*;

#[test]
fn runtime_15_ui_accessibility_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/accessibility.rs");
    let activation_actions = read_runtime_src("ui/tests/accessibility/activation_actions.rs");
    let description_references =
        read_runtime_src("ui/tests/accessibility/description_references.rs");
    let extraction = read_runtime_src("ui/tests/accessibility/extraction.rs");
    let focus_diagnostics = read_runtime_src("ui/tests/accessibility/focus_diagnostics.rs");
    let naming_relations = read_runtime_src("ui/tests/accessibility/naming_relations.rs");
    let value_actions = read_runtime_src("ui/tests/accessibility/value_actions.rs");

    assert_contains_all(
        "UI accessibility parent mounts folder-backed children",
        &parent,
        &[
            "mod activation_actions;",
            "mod description_references;",
            "mod extraction;",
            "mod focus_diagnostics;",
            "mod naming_relations;",
            "mod value_actions;",
            "fn root_surface()",
            "fn dispatch_accessibility(",
            "fn dispatch_accessibility_with_value(",
            "fn assert_widget_binding_report(",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/accessibility.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "extraction_includes_widget_only_contract_nodes",
        "name_priority_uses_explicit_labelled_by_text_alt_then_tooltip",
        "invalid_focus_falls_back_to_root_and_reports_diagnostic",
        "description_references_resolve_to_target_text",
        "accessibility_activate_uses_widget_toggle_behavior_alias",
        "accessibility_set_value_updates_editable_text_property",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI accessibility test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "accessibility extraction child owns snapshot extraction tests",
        &extraction,
        &[
            "fn extraction_includes_widget_only_contract_nodes",
            "fn extraction_infers_role_and_actions_from_authored_widget_behavior",
            "fn extraction_reads_value_state_from_runtime_component_state",
            "fn extraction_includes_interactive_text_alt_and_explicit_nodes",
        ],
    );
    assert_contains_all(
        "accessibility naming child owns naming and relation tests",
        &naming_relations,
        &[
            "fn name_priority_uses_explicit_labelled_by_text_alt_then_tooltip",
            "fn labelled_by_uses_higher_id_tooltip_target_without_order_dependency",
            "fn excluded_hidden_relation_owners_do_not_retain_targets",
            "fn hidden_excluded_containers_block_descendant_promotion",
        ],
    );
    assert_contains_all(
        "accessibility focus diagnostics child owns focus, disabled, and bounds tests",
        &focus_diagnostics,
        &[
            "fn focus_inside_hidden_subtree_falls_back_and_reports_excluded_focus",
            "fn disabled_nodes_are_discoverable_with_invalid_actions_filtered",
            "fn hidden_focusable_nodes_are_diagnosed_without_normal_inclusion",
            "fn missing_bounds_diagnostics_report_named_or_interactive_nodes",
        ],
    );
    assert_contains_all(
        "accessibility description child owns description and malformed-reference tests",
        &description_references,
        &[
            "fn description_references_resolve_to_target_text",
            "fn malformed_labelled_by_reports_invalid_label_reference",
            "fn hidden_widget_label_for_targets_are_not_retained_as_source_text_targets",
            "fn unsupported_role_actions_are_diagnosed",
        ],
    );
    assert_contains_all(
        "accessibility activation child owns focus, activate, hidden, and excluded actions",
        &activation_actions,
        &[
            "fn accessibility_focus_action_changes_runtime_focus",
            "fn accessibility_activate_emits_default_commit_component_event",
            "fn accessibility_activate_uses_widget_toggle_behavior_alias",
            "fn accessibility_visible_excluded_target_rejects_without_component_or_property_mutation",
        ],
    );
    assert_contains_all(
        "accessibility value child owns increment, set-value, dismiss, and text editing actions",
        &value_actions,
        &[
            "fn accessibility_increment_and_decrement_step_slider_value",
            "fn accessibility_set_value_uses_widget_value_property_alias",
            "fn accessibility_dismiss_requires_popup_id",
            "fn accessibility_set_value_updates_editable_text_property",
        ],
    );

    let child_test_total = [
        activation_actions.as_str(),
        description_references.as_str(),
        extraction.as_str(),
        focus_diagnostics.as_str(),
        naming_relations.as_str(),
        value_actions.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 49,
        "UI accessibility children should preserve all 49 parent tests"
    );

    for (path, source) in [
        ("ui/tests/accessibility.rs", parent.as_str()),
        (
            "ui/tests/accessibility/activation_actions.rs",
            activation_actions.as_str(),
        ),
        (
            "ui/tests/accessibility/description_references.rs",
            description_references.as_str(),
        ),
        ("ui/tests/accessibility/extraction.rs", extraction.as_str()),
        (
            "ui/tests/accessibility/focus_diagnostics.rs",
            focus_diagnostics.as_str(),
        ),
        (
            "ui/tests/accessibility/naming_relations.rs",
            naming_relations.as_str(),
        ),
        (
            "ui/tests/accessibility/value_actions.rs",
            value_actions.as_str(),
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
                "Runtime 15 M3 UI accessibility test folder split",
                "runtime_15_ui_accessibility_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/accessibility.rs",
                "ui/tests/accessibility/extraction.rs",
                "ui/tests/accessibility/value_actions.rs",
                "runtime_15_ui_accessibility_tests_are_folder_backed",
            ],
        );
    }
}
