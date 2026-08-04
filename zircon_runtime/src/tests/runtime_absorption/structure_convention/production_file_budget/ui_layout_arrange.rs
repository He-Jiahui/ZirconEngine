use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_ui_layout_arrange_grid_masonry_is_child_owner() {
    let parent = read_runtime_src("ui/layout/pass/arrange.rs");
    let grid_masonry = read_runtime_src("ui/layout/pass/arrange/grid_masonry.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");

    assert_contains_all(
        "layout arrange parent keeps entry, non-grid layout families, and child mount",
        &parent,
        &[
            "mod grid_masonry;",
            "use self::grid_masonry::{arrange_grid_children, arrange_masonry_children};",
            "pub(crate) fn arrange_node(",
            "fn arrange_size_box_children(",
            "fn arrange_block_children(",
            "fn arrange_linear_children(",
            "fn arrange_scrollable_children(",
            "fn arrange_wrap_children(",
            "fn child_positions(",
            "pub(super) fn hide_subtree_layout(",
        ],
    );
    for moved_owner in [
        "fn arrange_grid_children(",
        "fn arrange_masonry_children(",
        "fn grid_dimensions(",
        "fn grid_placement_for_child(",
        "fn grid_cell_frame(",
        "fn masonry_child_outer_height(",
        "fn masonry_target_column(",
        "UiGridSlotPlacement",
        "UiMasonryBoxConfig",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "ui/layout/pass/arrange.rs should delegate grid/masonry owner `{moved_owner}` to grid_masonry.rs"
        );
    }

    assert_contains_all(
        "layout arrange grid/masonry child owns placement and masonry column helpers",
        &grid_masonry,
        &[
            "pub(super) fn arrange_grid_children(",
            "pub(super) fn arrange_masonry_children(",
            "fn grid_dimensions(",
            "fn grid_placement_for_child(",
            "fn grid_cell_frame(",
            "fn masonry_child_outer_height(",
            "fn masonry_target_column(",
            "UiGridSlotPlacement",
            "UiMasonryBoxConfig",
            "free_child_frame",
            "ordered_children_for_container",
            "hide_subtree_layout",
            "arrange_node",
        ],
    );

    for (path, source) in [
        ("ui/layout/pass/arrange.rs", parent.as_str()),
        (
            "ui/layout/pass/arrange/grid_masonry.rs",
            grid_masonry.as_str(),
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
                "Runtime 15 M4 UI layout arrange grid/masonry owner split",
                "runtime_15_ui_layout_arrange_grid_masonry_owner_split_static_passed_cargo_deferred",
                "ui/layout/pass/arrange.rs",
                "ui/layout/pass/arrange/grid_masonry.rs",
                "runtime_15_ui_layout_arrange_grid_masonry_is_child_owner",
            ],
        );
    }
}
