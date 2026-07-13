use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_plugin_importer_dx_structure_assertions_guard_folder_backed_status_is_current() {
    let status_rows = read_runtime_src(REVIEW_GUARD_STATUS_ROWS_PATH);
    let status_map = read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH);
    let date_map = read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (label, source) in [
        ("plugin-importer row data", status_rows.as_str()),
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
                PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_GUARD_SPLIT_NAME,
                PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_GUARD_SPLIT_ID,
                PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD,
                PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_CHILD,
                PLUGIN_IMPORTER_DX_STRUCTURE_DELEGATION_CHILD,
                PLUGIN_IMPORTER_DX_STRUCTURE_CHILD_OWNERSHIP_CHILD,
                PLUGIN_IMPORTER_DX_STRUCTURE_STATUS_MIRRORS_CHILD,
                PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_CHILD,
                "runtime_15_plugin_importer_dx_structure_assertions_are_child_owner",
                "runtime_15_plugin_importer_dx_structure_assertions_children_are_child_owned",
                "runtime_15_plugin_importer_d13_sdk_structure_assertions_are_child_owner",
                "runtime_15_plugin_importer_dx_structure_assertions_guard_folder_backed_status_is_current",
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records plugin-importer DX structure assertions folder-backed split",
        &status_map,
        &[
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_GUARD_SPLIT_NAME,
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_GUARD_SPLIT_ID,
        ],
    );
    assert_contains_all(
        "M3 review date map records plugin-importer DX structure assertions folder-backed split",
        &date_map,
        &[
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_GUARD_SPLIT_NAME,
            "2026-07-02",
        ],
    );

    let parent = read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD);
    for (path, source) in [(PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD, parent)]
        .into_iter()
        .chain(plugin_importer_dx_structure_assertion_child_sources())
    {
        let line_count = source.lines().count();
        assert!(
            line_count < PLUGIN_IMPORTER_DX_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
