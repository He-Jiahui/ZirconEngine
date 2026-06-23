use super::*;

mod facade;
mod material;
mod pack;
mod project;

#[test]
fn runtime_15_asset_test_budget_guard_child_owner_split() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests.rs",
    );
    let pack = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/pack.rs",
    );
    let facade = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/facade.rs",
    );
    let project = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/project.rs",
    );
    let material = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/material.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );

    assert_contains_all(
        "asset test-budget parent mounts child owners",
        &parent,
        &[
            "mod facade;",
            "mod material;",
            "mod pack;",
            "mod project;",
            "fn runtime_15_asset_test_budget_guard_child_owner_split",
        ],
    );

    let asset_pack_guard = format!(
        "{}{}",
        "fn runtime_15_asset", "_pack_tests_are_folder_backed"
    );
    let asset_facade_guard = format!(
        "{}{}",
        "fn runtime_15_asset", "_facade_tests_are_folder_backed"
    );
    let asset_project_zmeta_guard = format!(
        "{}{}",
        "fn runtime_15_asset_project", "_zmeta_tests_are_folder_backed"
    );
    let asset_project_manager_guard = format!(
        "{}{}",
        "fn runtime_15_asset_project", "_manager_tests_are_folder_backed"
    );
    let asset_material_guard = format!(
        "{}{}",
        "fn runtime_15_asset", "_material_tests_are_folder_backed"
    );

    for moved_guard in [
        asset_pack_guard.as_str(),
        asset_facade_guard.as_str(),
        asset_project_zmeta_guard.as_str(),
        asset_project_manager_guard.as_str(),
        asset_material_guard.as_str(),
    ] {
        assert!(
            !parent.contains(moved_guard),
            "asset_tests.rs should mount child guard owners instead of defining {moved_guard}"
        );
    }

    assert_contains_all(
        "asset pack test-budget child owns pack guard",
        &pack,
        &[
            "use super::*;",
            asset_pack_guard.as_str(),
            "asset/tests/pack/delta_installer.rs",
        ],
    );
    assert_contains_all(
        "asset facade test-budget child owns facade guard",
        &facade,
        &[
            "use super::*;",
            asset_facade_guard.as_str(),
            "asset/tests/facade/recursive_dependencies.rs",
        ],
    );
    assert_contains_all(
        "asset project test-budget child owns project guards",
        &project,
        &[
            "use super::*;",
            asset_project_zmeta_guard.as_str(),
            asset_project_manager_guard.as_str(),
            "asset/tests/project/zmeta/compound_shader.rs",
            "asset/tests/project/manager/restore_failure_migration.rs",
        ],
    );
    assert_contains_all(
        "asset material test-budget child owns material guard",
        &material,
        &[
            "use super::*;",
            asset_material_guard.as_str(),
            "asset/tests/assets/material/owned_descriptor.rs",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests.rs",
            parent.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/pack.rs",
            pack.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/facade.rs",
            facade.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/project.rs",
            project.as_str(),
        ),
        (
            "tests/runtime_absorption/structure_convention/test_file_budget/asset_tests/material.rs",
            material.as_str(),
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
                "Runtime 15 M3 asset test-budget guard child-owner split",
                "runtime_15_asset_test_budget_guard_child_owner_split_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/asset_tests/pack.rs",
                "structure_convention/test_file_budget/asset_tests/project.rs",
                "runtime_15_asset_test_budget_guard_child_owner_split",
            ],
        );
    }
}
