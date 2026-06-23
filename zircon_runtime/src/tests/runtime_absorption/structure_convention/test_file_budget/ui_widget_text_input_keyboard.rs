use super::*;

#[test]
fn runtime_15_ui_widget_text_input_keyboard_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/widget_text_input_keyboard.rs");
    let basic_editing = read_runtime_src("ui/tests/widget_text_input_keyboard/basic_editing.rs");
    let clipboard_newline =
        read_runtime_src("ui/tests/widget_text_input_keyboard/clipboard_newline.rs");
    let selection_navigation =
        read_runtime_src("ui/tests/widget_text_input_keyboard/selection_navigation.rs");
    let text_ime = read_runtime_src("ui/tests/widget_text_input_keyboard/text_ime.rs");
    let word_shortcuts = read_runtime_src("ui/tests/widget_text_input_keyboard/word_shortcuts.rs");

    assert_contains_all(
        "UI widget text input keyboard parent mounts folder-backed children and keeps helpers",
        &parent,
        &[
            "mod basic_editing;",
            "mod clipboard_newline;",
            "mod selection_navigation;",
            "mod text_ime;",
            "mod word_shortcuts;",
            "fn dispatch_key(",
            "fn dispatch_text(",
            "fn dispatch_ime(",
            "fn text_input_surface(",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/widget_text_input_keyboard.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "text_input_keyboard_backspace_uses_widget_value_property",
        "text_input_keyboard_shift_arrow_left_extends_selection_without_value_event",
        "text_input_keyboard_control_backspace_deletes_previous_word",
        "text_input_keyboard_control_c_requests_clipboard_write_for_selection",
        "text_input_ime_commit_replaces_composition_and_emits_commit_event",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI widget text input keyboard test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI widget text input keyboard basic-editing child owns edit and grapheme tests",
        &basic_editing,
        &[
            "fn text_input_keyboard_backspace_uses_widget_value_property",
            "fn text_input_keyboard_read_only_backspace_does_not_mutate_value",
            "fn text_input_keyboard_backspace_deletes_previous_grapheme_cluster",
            "fn text_input_keyboard_delete_removes_next_grapheme_cluster",
        ],
    );
    assert_contains_all(
        "UI widget text input keyboard selection child owns navigation tests",
        &selection_navigation,
        &[
            "fn text_input_keyboard_shift_arrow_left_extends_selection_without_value_event",
            "fn text_input_keyboard_home_moves_to_current_line_start",
            "fn text_input_keyboard_arrow_down_handles_crlf_boundaries",
            "fn text_input_keyboard_control_shift_arrow_down_extends_to_document_end",
        ],
    );
    assert_contains_all(
        "UI widget text input keyboard word-shortcut child owns word and escape tests",
        &word_shortcuts,
        &[
            "fn text_input_keyboard_control_arrow_right_moves_to_word_end",
            "fn text_input_keyboard_control_backspace_deletes_previous_word",
            "fn text_input_keyboard_control_a_selects_all_text",
            "fn text_input_keyboard_escape_cancels_composition_before_selection_collapse",
        ],
    );
    assert_contains_all(
        "UI widget text input keyboard clipboard child owns clipboard and newline tests",
        &clipboard_newline,
        &[
            "fn text_input_keyboard_control_c_requests_clipboard_write_for_selection",
            "fn text_input_keyboard_paste_key_requests_clipboard_read",
            "fn text_input_keyboard_enter_inserts_newline_when_multiline",
            "fn text_input_keyboard_enter_respects_explicit_single_line",
        ],
    );
    assert_contains_all(
        "UI widget text input keyboard text/IME child owns text and composition tests",
        &text_ime,
        &[
            "fn text_input_text_event_replaces_active_selection",
            "fn text_input_selection_replacement_respects_max_chars",
            "fn text_input_ime_preedit_replaces_active_selection_and_tracks_composition",
            "fn text_input_ime_commit_replaces_composition_and_emits_commit_event",
        ],
    );

    let child_test_total = [
        basic_editing.as_str(),
        clipboard_newline.as_str(),
        selection_navigation.as_str(),
        text_ime.as_str(),
        word_shortcuts.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 52,
        "UI widget text input keyboard children should preserve all 52 parent tests"
    );

    for (path, source) in [
        ("ui/tests/widget_text_input_keyboard.rs", parent.as_str()),
        (
            "ui/tests/widget_text_input_keyboard/basic_editing.rs",
            basic_editing.as_str(),
        ),
        (
            "ui/tests/widget_text_input_keyboard/clipboard_newline.rs",
            clipboard_newline.as_str(),
        ),
        (
            "ui/tests/widget_text_input_keyboard/selection_navigation.rs",
            selection_navigation.as_str(),
        ),
        (
            "ui/tests/widget_text_input_keyboard/text_ime.rs",
            text_ime.as_str(),
        ),
        (
            "ui/tests/widget_text_input_keyboard/word_shortcuts.rs",
            word_shortcuts.as_str(),
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
                "Runtime 15 M3 UI widget text input keyboard test folder split",
                "runtime_15_ui_widget_text_input_keyboard_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/widget_text_input_keyboard.rs",
                "ui/tests/widget_text_input_keyboard/basic_editing.rs",
                "ui/tests/widget_text_input_keyboard/selection_navigation.rs",
                "ui/tests/widget_text_input_keyboard/text_ime.rs",
                "runtime_15_ui_widget_text_input_keyboard_tests_are_folder_backed",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M3 UI widget text input keyboard test folder split",
            "runtime_15_ui_widget_text_input_keyboard_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/widget_text_input_keyboard.rs",
            "ui/tests/widget_text_input_keyboard/basic_editing.rs",
            "runtime_15_ui_widget_text_input_keyboard_tests_are_folder_backed",
        ],
    );
}
