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
    let status_scan = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/status_scan.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/test_file_budget.rs",
    );

    assert_contains_all(
        "root-layout parent mounts child guard owners",
        &parent,
        &[
            "#[path = \"root_layout/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"root_layout/module_layout.rs\"]",
            "mod module_layout;",
            "#[path = \"root_layout/status_scan.rs\"]",
            "mod status_scan;",
            "#[path = \"root_layout/ui_children.rs\"]",
            "mod ui_children;",
        ],
    );
    for moved_anchor in [
        "runtime_15_test_file_budget_guard_is_folder_backed",
        "runtime_15_test_file_budget_root_layout_child_split_static_passed_cargo_deferred",
        "runtime_15_test_file_budget_root_layout_status_scan_child_split_static_passed_cargo_deferred",
        "runtime_15_test_file_budget_root_layout_status_scan_is_child_owner",
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
    assert_contains_all(
        "root-layout status scan child tracks new root-layout children",
        &status_scan,
        &[
            "structure_convention/test_file_budget/root_layout/folder_backed.rs",
            "structure_convention/test_file_budget/root_layout/module_layout.rs",
            "runtime_15_test_file_budget_root_layout_folder_backed_guard_child_split_static_passed_cargo_timeout_no_result",
            "runtime_15_test_file_budget_root_layout_folder_backed_guard_is_child_owner",
        ],
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
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/status_scan.rs",
            status_scan.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 test file budget root-layout folder-backed guard child split",
                "runtime_15_test_file_budget_root_layout_folder_backed_guard_child_split_static_passed_cargo_timeout_no_result",
                "structure_convention/test_file_budget/root_layout.rs",
                "structure_convention/test_file_budget/root_layout/folder_backed.rs",
                "structure_convention/test_file_budget/root_layout/module_layout.rs",
                "runtime_15_test_file_budget_root_layout_folder_backed_guard_is_child_owner",
            ],
        );
    }
}
