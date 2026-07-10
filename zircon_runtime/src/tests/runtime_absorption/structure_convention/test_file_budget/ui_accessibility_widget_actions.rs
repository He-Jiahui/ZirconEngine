use super::*;

#[test]
fn runtime_15_ui_accessibility_widget_actions_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/accessibility_widget_actions.rs");
    let disclosure_actions =
        read_runtime_src("ui/tests/accessibility_widget_actions/disclosure_actions.rs");
    let popup_actions = read_runtime_src("ui/tests/accessibility_widget_actions/popup_actions.rs");
    let tooltip_menu = read_runtime_src("ui/tests/accessibility_widget_actions/tooltip_menu.rs");

    assert_contains_all(
        "UI accessibility widget-actions parent mounts folder-backed children",
        &parent,
        &[
            "mod disclosure_actions;",
            "mod popup_actions;",
            "mod tooltip_menu;",
            "fn root_surface()",
            "fn dispatch_accessibility(",
            "fn assert_accessibility_binding_report(",
            "fn assert_widget_binding_report(",
            "fn insert_runtime_open_widget(",
            "fn insert_runtime_popup_dialog(",
            "fn insert_runtime_tooltip(",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/accessibility_widget_actions.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "extraction_reads_expanded_state_from_runtime_component_open_alias",
        "accessibility_activate_uses_runtime_component_popup_open_alias",
        "popup_dialog_default_actions_expose_dismiss_without_expand_collapse",
        "accessibility_dismiss_hides_active_runtime_tooltip",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI accessibility widget-action test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI accessibility widget disclosure child owns disclosure contracts",
        &disclosure_actions,
        &[
            "fn extraction_reads_expanded_state_from_runtime_component_open_alias",
            "fn accessibility_activate_uses_runtime_component_open_alias",
            "fn accessibility_expand_sets_runtime_component_disclosure_alias",
        ],
    );
    assert_contains_all(
        "UI accessibility widget popup child owns popup contracts",
        &popup_actions,
        &[
            "fn extraction_reads_popup_state_from_runtime_component_open_alias",
            "fn accessibility_activate_uses_runtime_component_popup_open_alias",
            "fn accessibility_collapse_sets_runtime_component_popup_open_alias",
            "fn accessibility_dismiss_closes_runtime_component_popup_open_alias",
            "fn popup_dialog_default_actions_expose_dismiss_without_expand_collapse",
            "fn popup_menu_default_actions_expose_expand_collapse_without_activate",
        ],
    );
    assert_contains_all(
        "UI accessibility widget tooltip/menu child owns tooltip and menu contracts",
        &tooltip_menu,
        &[
            "fn accessibility_dismiss_hides_active_runtime_tooltip",
            "fn accessibility_menu_item_activate_without_item_binding_closes_popup",
        ],
    );

    let child_test_total = [
        disclosure_actions.as_str(),
        popup_actions.as_str(),
        tooltip_menu.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 11,
        "UI accessibility widget-action children should preserve all 11 parent tests"
    );

    for (path, source) in [
        ("ui/tests/accessibility_widget_actions.rs", parent.as_str()),
        (
            "ui/tests/accessibility_widget_actions/disclosure_actions.rs",
            disclosure_actions.as_str(),
        ),
        (
            "ui/tests/accessibility_widget_actions/popup_actions.rs",
            popup_actions.as_str(),
        ),
        (
            "ui/tests/accessibility_widget_actions/tooltip_menu.rs",
            tooltip_menu.as_str(),
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
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 UI accessibility widget actions test folder split",
                "runtime_15_ui_accessibility_widget_actions_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/accessibility_widget_actions.rs",
                "ui/tests/accessibility_widget_actions/popup_actions.rs",
                "ui/tests/accessibility_widget_actions/tooltip_menu.rs",
                "runtime_15_ui_accessibility_widget_actions_tests_are_folder_backed",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M3 UI accessibility widget actions test folder split",
            "runtime_15_ui_accessibility_widget_actions_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/accessibility_widget_actions.rs",
            "ui/tests/accessibility_widget_actions/popup_actions.rs",
            "runtime_15_ui_accessibility_widget_actions_tests_are_folder_backed",
        ],
    );
}
