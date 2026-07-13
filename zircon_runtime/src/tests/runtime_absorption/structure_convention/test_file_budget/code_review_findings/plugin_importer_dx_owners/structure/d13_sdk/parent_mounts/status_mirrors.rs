use super::super::super::super::super::super::*;
use super::super::*;
use super::*;

const STRUCTURE_ASSERTION_STATUS_ROWS_PATH: &str = "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/plugin_importer_rows/structure_assertions.rs";

pub(super) fn assert_plugin_importer_d13_sdk_parent_mounts_status_mirrors_are_current() {
    let status_rows = read_runtime_src(STRUCTURE_ASSERTION_STATUS_ROWS_PATH);
    let status_map = read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH);
    let date_map = read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (label, source) in [
        ("plugin-importer DX status row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILD_SPLIT_SLICE,
                PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILD_SPLIT_STATUS,
                PLUGIN_IMPORTER_D13_PARENT_MOUNTS_DELEGATION_CHILD,
                PLUGIN_IMPORTER_D13_PARENT_MOUNTS_REVIEW_MOUNTS_CHILD,
                PLUGIN_IMPORTER_D13_PARENT_MOUNTS_FOLDER_BACKED_CHILD,
                PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILD_OWNERSHIP_CHILD,
                PLUGIN_IMPORTER_D13_PARENT_MOUNTS_STATUS_MIRRORS_CHILD,
                PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILD_SPLIT_GUARD,
                PLUGIN_IMPORTER_D13_PARENT_MOUNTS_STATUS_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records plugin-importer D13 parent-mount child split",
        &status_map,
        &[
            PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILD_SPLIT_SLICE,
            PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILD_SPLIT_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records plugin-importer D13 parent-mount child split",
        &date_map,
        &[
            PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILD_SPLIT_SLICE,
            PLUGIN_IMPORTER_D13_PARENT_MOUNTS_CHILD_SPLIT_DATE,
        ],
    );
}

#[test]
fn runtime_15_plugin_importer_d13_sdk_parent_mounts_status_mirrors_are_current() {
    assert_plugin_importer_d13_sdk_parent_mounts_status_mirrors_are_current();
}
