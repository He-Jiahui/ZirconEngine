use super::super::super::super::super::super::*;
use super::super::super::*;

pub(in super::super) fn assert_typed_error_source_inventory_child_sources_structure_guard_status_is_current(
) {
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
                TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_SPLIT,
                TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STATUS,
                TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_SPLIT,
                TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_STATUS,
                TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_ROUTE_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_STATUS_MIRRORS_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_BUDGETS_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_CHILD_BACKED_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records typed-error source inventory child_sources structure guard split",
        &status_map,
        &[
            TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_SPLIT,
            TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STATUS,
            TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_SPLIT,
            TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records typed-error source inventory child_sources structure guard split",
        &date_map,
        &[
            TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_SPLIT,
            TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_DATE,
            TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_SPLIT,
            TYPED_ERROR_SOURCE_INVENTORY_CHILD_SOURCES_STRUCTURE_GUARD_DATE,
        ],
    );
}
