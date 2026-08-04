use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_ui_text_layout_engine_visual_order_is_child_owner() {
    let parent = read_runtime_src("ui/text/layout_engine.rs");
    let visual_order = read_runtime_src("ui/text/layout_engine/visual_order.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");

    assert_contains_all(
        "UI text layout engine parent keeps layout entry points and shared helpers",
        &parent,
        &[
            "mod visual_order;",
            "visual_order::apply_visual_order(line, visible_text, direction);",
            "pub(crate) fn measure_text_size",
            "pub(crate) fn layout_text",
            "mod range_mapping;",
            "mod wrapping;",
            "use wrapping::wrap_source_runs_with_provider;",
            "text_advance, MIN_TEXT_FONT_SIZE",
            "#[cfg(test)]",
            "mod tests;",
        ],
    );
    for moved_owner in [
        "struct VisualTextToken",
        "struct VisualTextCluster",
        "struct VisualTextFragment",
        "fn visual_text_fragments(",
        "fn neutral_token_direction(",
        "fn push_visual_fragment(",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "ui/text/layout_engine.rs should delegate visual-order owner `{moved_owner}` to visual_order.rs"
        );
    }
    assert_contains_all(
        "UI text layout visual-order child adapts the shared UAX#9 line owner",
        &visual_order,
        &[
            "pub(super) fn apply_visual_order",
            "struct VisualTextToken",
            "struct VisualTextCluster",
            "struct VisualTextFragment",
            "fn logical_text_clusters(",
            "analyze_bidi_line",
            "mirrored_bidi_char",
            "fn push_visual_fragment(",
            "source_subrange",
        ],
    );

    for (path, source) in [
        ("ui/text/layout_engine.rs", parent.as_str()),
        (
            "ui/text/layout_engine/visual_order.rs",
            visual_order.as_str(),
        ),
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
                "Runtime 15 M4 UI text layout engine visual-order owner split",
                "runtime_15_ui_text_layout_engine_visual_order_owner_split_static_passed_cargo_deferred",
                "ui/text/layout_engine.rs",
                "ui/text/layout_engine/visual_order.rs",
                "runtime_15_ui_text_layout_engine_visual_order_is_child_owner",
            ],
        );
    }
}
