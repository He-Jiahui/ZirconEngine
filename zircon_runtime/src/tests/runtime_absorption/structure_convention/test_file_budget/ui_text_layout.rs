use super::*;

const STATUS: &str =
    "runtime_15_ui_text_layout_folder_backed_owner_split_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 UI text layout folder-backed owner split";
const GUARD: &str = "runtime_15_ui_text_layout_tests_are_folder_backed";

#[test]
fn runtime_15_ui_text_layout_tests_are_folder_backed() {
    assert!(
        !runtime_src_path("ui/tests/text_layout.rs").exists(),
        "flat ui/tests/text_layout.rs should stay deleted after folder-backed owner split"
    );

    let parent = read_runtime_src("ui/tests/text_layout/mod.rs");
    let alignment = read_runtime_src("ui/tests/text_layout/alignment.rs");
    let wrapping = read_runtime_src("ui/tests/text_layout/wrapping.rs");
    let overflow = read_runtime_src("ui/tests/text_layout/overflow.rs");
    let direction = read_runtime_src("ui/tests/text_layout/direction.rs");
    let edit_state = read_runtime_src("ui/tests/text_layout/edit_state.rs");

    assert_contains_all(
        "UI text layout parent is a navigational folder-backed owner",
        &parent,
        &[
            "mod alignment;",
            "mod direction;",
            "mod edit_state;",
            "mod overflow;",
            "mod wrapping;",
            "fn first_text_layout(",
            "fn first_text_layout_command(",
        ],
    );

    assert!(
        !parent.contains("#[test]"),
        "ui/tests/text_layout/mod.rs should stay navigational and keep tests in child owners"
    );

    for moved_test in [
        "fn render_extract_outputs_aligned_wrapped_text_layout",
        "fn render_extract_parses_justify_text_align_and_expands_non_final_line",
        "fn render_extract_parses_word_smart_wrap_layout",
        "fn render_extract_parses_word_smart_wrap_alias",
        "fn render_extract_parses_vertical_rl_writing_mode_layout",
        "fn render_extract_parses_start_ellipsis_overflow",
        "fn render_extract_parses_word_ellipsis_overflow",
        "fn render_extract_parses_middle_ellipsis_overflow",
        "fn render_extract_parses_shrink_to_fit_overflow_and_scales_font",
        "fn render_extract_parses_clamp_font_size_overflow_and_bounds",
        "fn render_extract_clips_text_layout_to_clip_frame",
        "fn render_extract_preserves_logical_start_text_align",
        "fn render_extract_auto_direction_uses_first_strong_for_logical_start_align",
        "fn render_extract_outputs_rich_directional_ellipsis_layout",
        "fn render_extract_outputs_visual_order_ranges_for_mixed_direction_text",
        "fn render_extract_keeps_neutral_separator_inside_rtl_visual_span",
        "fn render_extract_outputs_editable_text_state_for_text_fields",
        "fn render_extract_injects_preedit_span_without_document_value_mutation",
        "fn editable_text_state_applies_selection_and_composition_actions",
        "fn editable_text_state_restores_preedit_text_when_composition_is_canceled",
        "fn editable_text_state_updates_composition_against_preedit_base_text",
        "fn editable_text_state_inserts_preedit_without_consuming_text_for_empty_range",
    ] {
        assert!(
            !parent.contains(moved_test),
            "ui/tests/text_layout/mod.rs should mount child owners instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "UI text layout alignment child owns align/justify render-extract tests",
        &alignment,
        &[
            "use super::*;",
            "fn render_extract_outputs_aligned_wrapped_text_layout",
            "fn render_extract_parses_justify_text_align_and_expands_non_final_line",
        ],
    );
    assert_contains_all(
        "UI text layout wrapping child owns wrap/writing-mode tests",
        &wrapping,
        &[
            "use super::*;",
            "fn render_extract_parses_word_smart_wrap_layout",
            "fn render_extract_parses_word_smart_wrap_alias",
            "fn render_extract_parses_vertical_rl_writing_mode_layout",
        ],
    );
    assert_contains_all(
        "UI text layout overflow child owns overflow and clipping tests",
        &overflow,
        &[
            "use super::*;",
            "fn render_extract_parses_start_ellipsis_overflow",
            "fn render_extract_parses_word_ellipsis_overflow",
            "fn render_extract_parses_middle_ellipsis_overflow",
            "fn render_extract_parses_shrink_to_fit_overflow_and_scales_font",
            "fn render_extract_parses_clamp_font_size_overflow_and_bounds",
            "fn render_extract_clips_text_layout_to_clip_frame",
        ],
    );
    assert_contains_all(
        "UI text layout direction child owns BiDi and visual-order tests",
        &direction,
        &[
            "use super::*;",
            "fn render_extract_preserves_logical_start_text_align",
            "fn render_extract_auto_direction_uses_first_strong_for_logical_start_align",
            "fn render_extract_outputs_rich_directional_ellipsis_layout",
            "fn render_extract_outputs_visual_order_ranges_for_mixed_direction_text",
            "fn render_extract_keeps_neutral_separator_inside_rtl_visual_span",
        ],
    );
    assert_contains_all(
        "UI text layout edit-state child owns editable/preedit tests",
        &edit_state,
        &[
            "use super::*;",
            "UiEditableTextState",
            "UiTextEditAction",
            "fn render_extract_outputs_editable_text_state_for_text_fields",
            "fn render_extract_injects_preedit_span_without_document_value_mutation",
            "fn editable_text_state_applies_selection_and_composition_actions",
            "fn editable_text_state_restores_preedit_text_when_composition_is_canceled",
            "fn editable_text_state_updates_composition_against_preedit_base_text",
            "fn editable_text_state_inserts_preedit_without_consuming_text_for_empty_range",
        ],
    );

    for (path, source) in [
        ("ui/tests/text_layout/mod.rs", parent.as_str()),
        ("ui/tests/text_layout/alignment.rs", alignment.as_str()),
        ("ui/tests/text_layout/wrapping.rs", wrapping.as_str()),
        ("ui/tests/text_layout/overflow.rs", overflow.as_str()),
        ("ui/tests/text_layout/direction.rs", direction.as_str()),
        ("ui/tests/text_layout/edit_state.rs", edit_state.as_str()),
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
    let text_doc = read_repo("docs/zircon_runtime/ui/text.md");
}
