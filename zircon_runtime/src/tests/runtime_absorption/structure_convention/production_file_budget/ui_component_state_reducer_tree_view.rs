use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_ui_component_state_reducer_tree_view_editing_is_child_owner() {
    let parent = read_runtime_src("ui/component/state_reducer/tree_view.rs");
    let editing = read_runtime_src("ui/component/state_reducer/tree_view/editing.rs");
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
        "tree-view parent keeps navigation, selection, and expansion responsibilities",
        &parent,
        &[
            "mod editing;",
            "pub(super) use editing::{apply_begin_edit, apply_cancel_editing, apply_commit};",
            "pub(super) fn is_tree_view(",
            "pub(super) fn apply_keyboard_expand_collapse(",
            "pub(super) fn apply_toggle_expanded(",
            "pub(super) fn apply_select_option(",
            "fn set_focused_node_expanded(",
            "fn apply_multi_select_option(",
            "fn apply_single_select_option(",
            "fn ordered_node_ids(",
            "fn expanded_node_ids(",
            "fn selected_node_ids(",
        ],
    );
    for moved_owner in [
        "const EDITING_NODE_ID_PROPERTIES",
        "const EDITING_TEXT_PROPERTIES",
        "fn focused_edit_target(",
        "fn clear_editing_state(",
        "fn tree_is_editing(",
        "fn editing_node_id(",
        "fn tree_node_label(",
        "fn find_tree_node_label(",
        "fn editing_node_id_property(",
        "fn preferred_property(",
        "fn is_editing_text_property(",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "ui/component/state_reducer/tree_view.rs should delegate editing owner `{moved_owner}` to tree_view/editing.rs"
        );
    }

    assert_contains_all(
        "tree-view editing child owns rename/editing state transitions",
        &editing,
        &[
            "const EDITING_NODE_ID_PROPERTIES",
            "const EDITING_TEXT_PROPERTIES",
            "const EDITING_INDEX_PROPERTIES",
            "const RENAMED_NODE_ID_PROPERTIES",
            "const RENAMED_TEXT_PROPERTIES",
            "const RENAME_COMMITTED_PROPERTIES",
            "pub(in crate::ui::component::state_reducer) fn apply_begin_edit(",
            "pub(in crate::ui::component::state_reducer) fn apply_cancel_editing(",
            "pub(in crate::ui::component::state_reducer) fn apply_commit(",
            "fn focused_edit_target(",
            "fn clear_editing_state(",
            "fn tree_is_editing(",
            "fn editing_node_id(",
            "fn tree_node_label(",
            "fn find_tree_node_label(",
            "fn preferred_property(",
            "fn is_editing_text_property(",
            "super::ordered_node_ids(",
            "super::current_tree_index(",
            "super::NODE_PROPERTIES",
            "super::super::set_value(",
        ],
    );

    for (path, source) in [
        ("ui/component/state_reducer/tree_view.rs", parent.as_str()),
        (
            "ui/component/state_reducer/tree_view/editing.rs",
            editing.as_str(),
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
                "Runtime 15 M4 UI component state-reducer tree view editing owner split",
                "runtime_15_ui_component_state_reducer_tree_view_editing_owner_split_static_passed_cargo_deferred",
                "ui/component/state_reducer/tree_view.rs",
                "ui/component/state_reducer/tree_view/editing.rs",
                "runtime_15_ui_component_state_reducer_tree_view_editing_is_child_owner",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M4 UI component state-reducer tree view editing owner split",
            "runtime_15_ui_component_state_reducer_tree_view_editing_owner_split_static_passed_cargo_deferred",
            "ui/component/state_reducer/tree_view.rs",
            "ui/component/state_reducer/tree_view/editing.rs",
            "runtime_15_ui_component_state_reducer_tree_view_editing_is_child_owner",
        ],
    );
}
