use super::super::assert_contains_all;
use super::{dead_code_suppression_lines, read_repo, read_runtime_src};

#[test]
fn runtime_15_ui_text_edit_state_dead_code_suppression_cleanup() {
    let ui_text_mod = read_runtime_src("ui/text/mod.rs");
    let edit_state = read_runtime_src("ui/text/edit_state.rs");
    let text_input = read_runtime_src("ui/component/state_reducer/text_input.rs");
    let editable_text = read_runtime_src("ui/surface/input/editable_text.rs");
    let keyboard_clipboard = read_runtime_src("ui/surface/input/keyboard_clipboard.rs");
    let text_pointer = read_runtime_src("ui/surface/input/text_pointer.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ui_text_doc = read_repo("docs/zircon_runtime/ui/text.md");
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs",
    );
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation/core_rows.rs",
    );

    assert!(
        dead_code_suppression_lines(&ui_text_mod).is_empty(),
        "ui/text/mod.rs should keep edit_state live without cfg_attr/allow(dead_code)"
    );
    assert_contains_all(
        "ui text edit state production module",
        &ui_text_mod,
        &[
            "mod edit_state;",
            "pub(crate) use edit_state::apply_text_edit_action;",
        ],
    );
    assert_contains_all(
        "ui text edit state owner",
        &edit_state,
        &[
            "pub(crate) fn apply_text_edit_action",
            "UiTextEditAction::Insert { text }",
            "UiTextEditAction::SetComposition { range, text }",
            "replace_range_preserving_composition",
            "previous_grapheme_boundary",
            "next_grapheme_boundary",
        ],
    );
    assert_contains_all(
        "text input edit state reducer consumer",
        &text_input,
        &[
            "use crate::ui::text::apply_text_edit_action;",
            "let next_state = apply_text_edit_action(",
        ],
    );
    for (label, source) in [
        ("editable text input consumer", editable_text.as_str()),
        ("keyboard clipboard consumer", keyboard_clipboard.as_str()),
        ("text pointer consumer", text_pointer.as_str()),
    ] {
        assert_contains_all(label, source, &["apply_text_edit_action("]);
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("UI text doc", ui_text_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F12 UI text edit-state dead-code suppression cleanup",
                "runtime_15_ui_text_edit_state_dead_code_suppression_cleanup_static_passed_cargo_deferred",
                "ui/text/mod.rs",
                "ui/text/edit_state.rs",
                "runtime_15_ui_text_edit_state_dead_code_suppression_cleanup",
            ],
        );
    }
    assert_contains_all(
        "Runtime 15 status map",
        &status_map,
        &[
            "Runtime 15 F12 UI text edit-state dead-code suppression cleanup",
            "runtime_15_ui_text_edit_state_dead_code_suppression_cleanup_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 date map",
        &date_map,
        &[
            "Runtime 15 F12 UI text edit-state dead-code suppression cleanup",
            "2026-06-27",
        ],
    );
}
