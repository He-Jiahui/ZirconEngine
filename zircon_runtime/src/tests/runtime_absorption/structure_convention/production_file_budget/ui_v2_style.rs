use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_ui_v2_style_runtime_state_is_child_owner() {
    let parent = read_runtime_src("ui/v2/style.rs");
    let runtime_state = read_runtime_src("ui/v2/style/runtime_state.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );

    assert_contains_all(
        "UI v2 style parent keeps resolver/index entry points and selector matching",
        &parent,
        &[
            "mod runtime_state;",
            "use runtime_state::{",
            "collect_pseudo_states",
            "collect_runtime_pseudo_states",
            "dirty_for_runtime_style_delta",
            "merge_dirty_flags_into",
            "pub struct UiV2StyleResolver",
            "pub(crate) struct UiV2RuntimeStyleIndex",
            "fn resolve_with_rules(",
            "fn collect_rules(",
            "fn resolve_value_map(",
            "struct SelectorPathNode",
            "trait UiV2SelectorMatchExt",
            "fn matches_segment(",
        ],
    );
    for moved_owner in [
        "fn append_resolved_painter_state(",
        "fn painter_state_from_selector_states(",
        "fn painter_family_for_component(",
        "fn apply_retained_runtime_state_attributes(",
        "fn dirty_for_runtime_style_delta(",
        "fn is_render_only_style_key(",
        "UiPainterStyleSelector",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "ui/v2/style.rs should delegate runtime-state owner `{moved_owner}` to runtime_state.rs"
        );
    }
    assert_contains_all(
        "runtime-state child owns pseudo-state, retained-state, and dirty-delta helpers",
        &runtime_state,
        &[
            "pub(super) fn collect_pseudo_states",
            "pub(super) fn collect_runtime_pseudo_states",
            "fn append_resolved_painter_state(",
            "fn painter_state_from_selector_states(",
            "fn painter_family_for_component(",
            "fn is_retained_runtime_state(",
            "pub(super) fn apply_retained_runtime_state_attributes",
            "pub(super) fn dirty_for_runtime_style_delta",
            "pub(super) fn merge_dirty_flags_into",
            "UiPainterStyleSelector::resolved_state_for_family",
            "BTreeSet",
        ],
    );

    for (path, source) in [
        ("ui/v2/style.rs", parent.as_str()),
        ("ui/v2/style/runtime_state.rs", runtime_state.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

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
                "Runtime 15 M4 UI v2 style runtime-state owner split",
                "runtime_15_ui_v2_style_runtime_state_owner_split_static_passed_cargo_deferred",
                "ui/v2/style.rs",
                "ui/v2/style/runtime_state.rs",
                "runtime_15_ui_v2_style_runtime_state_is_child_owner",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M4 UI v2 style runtime-state owner split",
            "runtime_15_ui_v2_style_runtime_state_owner_split_static_passed_cargo_deferred",
            "ui/v2/style.rs",
            "ui/v2/style/runtime_state.rs",
            "runtime_15_ui_v2_style_runtime_state_is_child_owner",
        ],
    );
}
