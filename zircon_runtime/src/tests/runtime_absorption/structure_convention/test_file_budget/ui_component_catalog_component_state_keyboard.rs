use super::*;

#[test]
fn runtime_15_ui_component_catalog_component_state_keyboard_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/component_catalog/component_state/keyboard.rs");
    let action_selection =
        read_runtime_src("ui/tests/component_catalog/component_state/keyboard/action_selection.rs");
    let menu_navigation =
        read_runtime_src("ui/tests/component_catalog/component_state/keyboard/menu_navigation.rs");
    let numeric_controls =
        read_runtime_src("ui/tests/component_catalog/component_state/keyboard/numeric_controls.rs");
    let text_inputs =
        read_runtime_src("ui/tests/component_catalog/component_state/keyboard/text_inputs.rs");

    assert_contains_all(
        "UI component state keyboard parent mounts folder-backed children",
        &parent,
        &[
            "mod action_selection;",
            "mod menu_navigation;",
            "mod numeric_controls;",
            "mod text_inputs;",
            "fn menu_option(",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/component_catalog/component_state/keyboard.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "material_keyboard_action_activates_buttons_and_toggles_checked_controls",
        "material_keyboard_action_moves_menu_focus_without_committing_selection",
        "material_keyboard_text_appends_text_input_values_without_full_editing_policy",
        "material_keyboard_action_targets_range_slider_focused_thumb",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI component-state keyboard test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI component state keyboard action child owns selection actions",
        &action_selection,
        &[
            "fn material_keyboard_action_activates_buttons_and_toggles_checked_controls",
            "fn material_keyboard_action_moves_tabs_with_selection_following_focus",
            "fn material_keyboard_action_moves_grouped_selection_controls",
            "fn material_keyboard_action_skips_disabled_grouped_selection_options",
            "fn material_keyboard_action_toggles_multiple_toggle_button_group_focused_option",
        ],
    );
    assert_contains_all(
        "UI component state keyboard menu child owns menu navigation",
        &menu_navigation,
        &[
            "fn material_keyboard_action_moves_menu_focus_without_committing_selection",
            "fn material_keyboard_action_moves_tree_and_table_focus_by_index",
            "fn material_keyboard_text_moves_menu_focus_by_first_character_without_committing_selection",
            "fn material_keyboard_text_matches_menu_prefix_without_committing_selection",
            "fn material_keyboard_text_buffers_menu_prefix_across_key_events_until_expired",
        ],
    );
    assert_contains_all(
        "UI component state keyboard text child owns text-input edits",
        &text_inputs,
        &[
            "fn material_keyboard_text_appends_text_input_values_without_full_editing_policy",
            "fn material_keyboard_text_replaces_text_input_selection_and_updates_caret_state",
        ],
    );
    assert_contains_all(
        "UI component state keyboard numeric child owns numeric controls",
        &numeric_controls,
        &[
            "fn material_keyboard_action_steps_numeric_controls_and_closes_popup_controls",
            "fn material_keyboard_action_targets_range_slider_focused_thumb",
        ],
    );

    let child_test_total = [
        action_selection.as_str(),
        menu_navigation.as_str(),
        numeric_controls.as_str(),
        text_inputs.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 14,
        "UI component state keyboard children should preserve all 14 parent tests"
    );

    for (path, source) in [
        (
            "ui/tests/component_catalog/component_state/keyboard.rs",
            parent.as_str(),
        ),
        (
            "ui/tests/component_catalog/component_state/keyboard/action_selection.rs",
            action_selection.as_str(),
        ),
        (
            "ui/tests/component_catalog/component_state/keyboard/menu_navigation.rs",
            menu_navigation.as_str(),
        ),
        (
            "ui/tests/component_catalog/component_state/keyboard/numeric_controls.rs",
            numeric_controls.as_str(),
        ),
        (
            "ui/tests/component_catalog/component_state/keyboard/text_inputs.rs",
            text_inputs.as_str(),
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
                "Runtime 15 M3 UI component state keyboard test folder split",
                "runtime_15_ui_component_catalog_component_state_keyboard_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/component_catalog/component_state/keyboard.rs",
                "ui/tests/component_catalog/component_state/keyboard/action_selection.rs",
                "ui/tests/component_catalog/component_state/keyboard/menu_navigation.rs",
                "runtime_15_ui_component_catalog_component_state_keyboard_tests_are_folder_backed",
            ],
        );
    }
}
