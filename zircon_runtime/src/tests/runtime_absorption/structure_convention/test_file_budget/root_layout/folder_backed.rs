use super::*;

#[path = "folder_backed/assertions.rs"]
mod assertions;
#[path = "folder_backed/guard_names.rs"]
mod guard_names;
#[path = "folder_backed/sources.rs"]
mod sources;

const ASSERTIONS_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/folder_backed/assertions.rs";
const GUARD_NAMES_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/folder_backed/guard_names.rs";
const SOURCES_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/folder_backed/sources.rs";

#[test]
fn runtime_15_test_file_budget_guard_is_folder_backed() {
    let sources = sources::read_guard_sources();
    let guards = guard_names::guard_names();

    assertions::assert_test_file_budget_root_is_folder_backed(&sources, &guards);
}

#[test]
fn runtime_15_test_file_budget_root_layout_folder_backed_support_child_owners_are_folder_backed() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/folder_backed.rs",
    );
    let assertions = read_runtime_src(ASSERTIONS_OWNER);
    let guard_names = read_runtime_src(GUARD_NAMES_OWNER);
    let sources = read_runtime_src(SOURCES_OWNER);
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
        "folder-backed guard parent mounts support child owners",
        &parent,
        &[
            "#[path = \"folder_backed/assertions.rs\"]",
            "mod assertions;",
            "#[path = \"folder_backed/guard_names.rs\"]",
            "mod guard_names;",
            "#[path = \"folder_backed/sources.rs\"]",
            "mod sources;",
            "fn runtime_15_test_file_budget_guard_is_folder_backed",
            "fn runtime_15_test_file_budget_root_layout_folder_backed_support_child_owners_are_folder_backed",
        ],
    );
    for moved_anchor in [
        concat!("pub(super) struct ", "GuardSources"),
        concat!("pub(super) struct ", "GuardNames"),
        concat!(
            "pub(super) fn assert_test_file_budget_root_",
            "is_folder_backed"
        ),
        concat!("let asset_tests = ", "read_runtime_src("),
        concat!("let asset_pack_guard = ", "format!("),
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "root_layout/folder_backed.rs should mount support child owners instead of keeping {moved_anchor}"
        );
    }
    assert_contains_all(
        "folder-backed assertions child owns root-layout checks",
        &assertions,
        &[
            concat!(
                "pub(super) fn assert_test_file_budget_root_",
                "is_folder_backed"
            ),
            "test file budget parent mounts folder-backed guard owners",
            "status output row-data Runtime 15 child owns Runtime 15 row-data guard",
            "UI shared core test-budget child owns historical shared-core guard",
        ],
    );
    assert_contains_all(
        "folder-backed guard-names child owns generated moved guard anchors",
        &guard_names,
        &[
            concat!("pub(super) struct ", "GuardNames"),
            "pub(super) fn guard_names",
            "asset_pack_guard",
            "shader_prewarm_manifest_guard",
            "status_output_row_data_guard",
        ],
    );
    assert_contains_all(
        "folder-backed sources child owns source reads",
        &sources,
        &[
            concat!("pub(super) struct ", "GuardSources"),
            "pub(super) fn read_guard_sources",
            "test_file_budget/asset_tests/pack.rs",
            "test_file_budget/status_output_row_data/runtime_15_row_data.rs",
            "test_file_budget/ui_shared_core.rs",
        ],
    );

    for (path, source) in [
        (
            "structure_convention/test_file_budget/root_layout/folder_backed.rs",
            parent.as_str(),
        ),
        (
            "structure_convention/test_file_budget/root_layout/folder_backed/assertions.rs",
            assertions.as_str(),
        ),
        (
            "structure_convention/test_file_budget/root_layout/folder_backed/guard_names.rs",
            guard_names.as_str(),
        ),
        (
            "structure_convention/test_file_budget/root_layout/folder_backed/sources.rs",
            sources.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the focused Runtime 15 test-support budget; got {line_count} lines"
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
                "Runtime 15 M3 test file budget root-layout folder-backed support child-owner split",
                "runtime_15_test_file_budget_root_layout_folder_backed_support_child_owner_split_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/root_layout/folder_backed.rs",
                "structure_convention/test_file_budget/root_layout/folder_backed/assertions.rs",
                "structure_convention/test_file_budget/root_layout/folder_backed/guard_names.rs",
                "runtime_15_test_file_budget_root_layout_folder_backed_support_child_owners_are_folder_backed",
            ],
        );
    }
}
