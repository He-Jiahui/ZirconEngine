use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_typed_error_source_inventory_guard_folder_backed_status_is_current() {
    let status_rows = typed_error_source_inventory_status_rows_source();
    let status_map = typed_error_source_inventory_status_map_source();
    let date_map = typed_error_source_inventory_date_map_source();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (label, source) in [
        ("typed-error status-support row data", status_rows.as_str()),
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
                TYPED_ERROR_SOURCE_INVENTORY_FOLDER_BACKED_SLICE,
                TYPED_ERROR_SOURCE_INVENTORY_FOLDER_BACKED_STATUS,
                TYPED_ERROR_SOURCE_INVENTORY_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_PATHS_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_READS_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_BUDGETS_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_STATUS_MIRRORS_CHILD,
                "runtime_15_typed_error_source_inventory_is_child_owner",
                TYPED_ERROR_SOURCE_INVENTORY_FOLDER_BACKED_GUARD,
                TYPED_ERROR_SOURCE_INVENTORY_STATUS_GUARD,
                TYPED_ERROR_SOURCE_INVENTORY_BUDGET_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records typed-error source inventory folder-backed split",
        &status_map,
        &[
            TYPED_ERROR_SOURCE_INVENTORY_FOLDER_BACKED_SLICE,
            TYPED_ERROR_SOURCE_INVENTORY_FOLDER_BACKED_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records typed-error source inventory folder-backed split",
        &date_map,
        &[
            TYPED_ERROR_SOURCE_INVENTORY_FOLDER_BACKED_SLICE,
            TYPED_ERROR_SOURCE_INVENTORY_FOLDER_BACKED_DATE,
        ],
    );
}
