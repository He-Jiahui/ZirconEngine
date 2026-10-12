use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn secure_text_keyboard_commands_do_not_query_word_boundaries() {
    let route = read_runtime_src("ui/surface/input/editable_text.rs");
    let commands = read_runtime_src("ui/surface/input/text_keyboard/edit_actions.rs");
    let behavior = read_runtime_src("ui/tests/widget_text_input_keyboard/word_shortcuts.rs");
    let structure = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "editable route passes the canonical secure class into the sole keyboard command owner",
        &route,
        &[
            "let secure = editable_text_input_is_secure(surface, target);",
            "keyboard_text_edit_actions(&keyboard, &editable, secure)",
        ],
    );
    assert_contains_all(
        "secure control commands use hard-line boundaries before ordinary word navigation",
        &commands,
        &[
            "let secure_line_navigation = secure && keyboard.metadata.modifiers.control;",
            "let word_navigation = !secure && keyboard.metadata.modifiers.control;",
            "\"Backspace\" if secure_line_deletion => Some(delete_to_line_start_actions(state))",
            "\"Delete\" if secure_line_deletion => Some(delete_to_line_end_actions(state))",
            "line_start_boundary(&state.text, caret)",
            "line_end_boundary(&state.text, caret)",
        ],
    );
    let secure_branch = commands
        .find("\"Backspace\" if secure_line_deletion")
        .expect("secure deletion branch");
    let word_branch = commands
        .find("\"Backspace\" if word_navigation")
        .expect("ordinary word deletion branch");
    assert!(
        secure_branch < word_branch,
        "secure deletion must resolve before ordinary word navigation"
    );
    assert_contains_all(
        "behavior coverage locks navigation and deletion without exposing secure content",
        &behavior,
        &[
            "secure_text_control_arrows_do_not_reveal_word_boundaries",
            "secure_text_control_delete_uses_line_boundaries_instead_of_words",
            "diagnostics.secure_text_redacted",
        ],
    );
    assert_contains_all(
        "structure convention retains the Unreal password-command boundary",
        &structure,
        &[
            "Control plus",
            "uses hard-line boundaries",
            "never queries the source word-boundary",
        ],
    );
}
