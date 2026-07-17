use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_plugin_importer_dx_source_inventory_guard_folder_backed_status_is_current() {
    let status_rows = format!(
        "{}\n{}",
        read_runtime_src(REVIEW_GUARD_STATUS_ROWS_PATH),
        read_runtime_src(REVIEW_GUARD_SOURCE_INVENTORY_STATUS_ROWS_PATH)
    );
    let status_map = read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH);
    let date_map = read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo(
            "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
        );
    let runtime_index = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
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
                PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_FOLDER_BACKED_SLICE,
                PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_FOLDER_BACKED_STATUS,
                PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_CHILD,
                PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_PATHS_CHILD,
                PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_READS_CHILD,
                PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_BUDGETS_CHILD,
                PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_DELEGATION_CHILD,
                PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_STATUS_MIRRORS_CHILD,
                "runtime_15_plugin_importer_dx_source_inventory_is_child_owner",
                PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_FOLDER_BACKED_GUARD,
                PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_STATUS_GUARD,
                PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_BUDGET_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records plugin-importer DX source inventory folder-backed split",
        &status_map,
        &[
            PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_FOLDER_BACKED_SLICE,
            PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_FOLDER_BACKED_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records plugin-importer DX source inventory folder-backed split",
        &date_map,
        &[
            PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_FOLDER_BACKED_SLICE,
            PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_FOLDER_BACKED_DATE,
        ],
    );
}
