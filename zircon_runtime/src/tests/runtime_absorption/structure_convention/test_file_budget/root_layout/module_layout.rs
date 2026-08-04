use super::*;

#[test]
fn runtime_15_test_file_budget_root_layout_folder_backed_guard_is_child_owner() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/root_layout.rs",
    );
    let folder_backed = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/folder_backed.rs",
    );
    let module_layout = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/module_layout.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "root-layout parent mounts child guard owners",
        &parent,
        &[
            "#[path = \"root_layout/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"root_layout/module_layout.rs\"]",
            "mod module_layout;",
            "#[path = \"root_layout/ui_children.rs\"]",
            "mod ui_children;",
        ],
    );
    for moved_anchor in [
        "runtime_15_test_file_budget_guard_is_folder_backed",
        "runtime_15_test_file_budget_root_layout_child_split_static_passed_cargo_deferred",
        "runtime_15_test_file_budget_root_layout_status_scan_child_split_static_passed_cargo_deferred",
        "runtime_15_test_file_budget_root_layout_ui_child_split_static_passed_cargo_deferred",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "test_file_budget/root_layout.rs should mount child guard owners instead of keeping {moved_anchor}"
        );
    }
    assert_contains_all(
        "root-layout folder-backed child owns original root guard",
        &folder_backed,
        &["fn runtime_15_test_file_budget_guard_is_folder_backed"],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/root_layout.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/folder_backed.rs",
            folder_backed.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/module_layout.rs",
            module_layout.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
