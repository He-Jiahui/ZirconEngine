use super::*;

#[test]
fn runtime_15_test_file_budget_parent_guard_child_owner_split() {
    let parent =
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/mod.rs");
    let core_framework = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/core_framework.rs",
    );
    let ui_v2_asset = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_v2_asset.rs",
    );
    let ui_shared_core = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_shared_core.rs",
    );
    let module_layout = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/module_layout.rs",
    );
    let root_layout_folder = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/folder_backed.rs",
    );
    let root_layout_status_scan = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/status_scan.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs",
    );

    assert_contains_all(
        "test-file budget parent mounts final child guard owners",
        &parent,
        &[
            "mod core_framework;",
            "mod module_layout;",
            "mod ui_shared_core;",
            "mod ui_v2_asset;",
            "fn read_runtime_src",
            "fn read_repo",
        ],
    );
    for moved_guard in [
        "fn runtime_15_core_framework_tests_are_folder_backed",
        "fn runtime_15_ui_v2_asset_tests_are_folder_backed",
        "fn runtime_15_ui_shared_core_tests_are_folder_backed",
    ] {
        assert!(
            !parent.contains(moved_guard),
            "test_file_budget/mod.rs should mount child guard owners instead of defining {moved_guard}"
        );
    }

    assert_contains_all(
        "core framework budget child owns the core framework guard",
        &core_framework,
        &[
            "use super::*;",
            "fn runtime_15_core_framework_tests_are_folder_backed",
        ],
    );
    assert_contains_all(
        "UI v2 asset budget child owns the UI v2 asset guard",
        &ui_v2_asset,
        &[
            "use super::*;",
            "fn runtime_15_ui_v2_asset_tests_are_folder_backed",
        ],
    );
    assert_contains_all(
        "UI shared core budget child owns the UI shared core guard",
        &ui_shared_core,
        &[
            "use super::*;",
            "fn runtime_15_ui_shared_core_tests_are_folder_backed",
        ],
    );
    assert_contains_all(
        "root-layout folder-backed guard tracks the new child owners",
        &root_layout_folder,
        &[
            "structure_convention/test_file_budget/core_framework.rs",
            "structure_convention/test_file_budget/ui_v2_asset.rs",
            "structure_convention/test_file_budget/ui_shared_core.rs",
            "fn runtime_15_test_file_budget_parent_guard_child_owner_split",
        ],
    );
    assert_contains_all(
        "root-layout status scan counts the new child owners",
        &root_layout_status_scan,
        &[
            "structure_convention/test_file_budget/core_framework.rs",
            "structure_convention/test_file_budget/module_layout.rs",
            "structure_convention/test_file_budget/ui_v2_asset.rs",
            "structure_convention/test_file_budget/ui_shared_core.rs",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/mod.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/core_framework.rs",
            core_framework.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/ui_v2_asset.rs",
            ui_v2_asset.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/ui_shared_core.rs",
            ui_shared_core.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/module_layout.rs",
            module_layout.as_str(),
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
                "Runtime 15 M3 test file budget parent guard child-owner split",
                "runtime_15_test_file_budget_parent_guard_child_owner_split_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/mod.rs",
                "structure_convention/test_file_budget/core_framework.rs",
                "structure_convention/test_file_budget/ui_v2_asset.rs",
                "structure_convention/test_file_budget/ui_shared_core.rs",
                "structure_convention/test_file_budget/module_layout.rs",
                "runtime_15_test_file_budget_parent_guard_child_owner_split",
            ],
        );
    }
}
