use super::super::super::super::super::super::*;
use super::super::*;

pub(super) fn assert_typed_error_source_inventory_metadata_status_current_mirrors_are_current() {
    let status_rows = read_runtime_src(REVIEW_GUARD_STATUS_ROWS_PATH);
    let status_map = format!(
        "{}\n{}",
        read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH),
        read_runtime_src(REVIEW_GUARD_TYPED_ERROR_STATUS_MAP_PATH)
    );
    let date_map = format!(
        "{}\n{}",
        read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH),
        read_runtime_src(REVIEW_GUARD_TYPED_ERROR_DATE_MAP_PATH)
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    for (label, source) in [
        ("typed-error status-support row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("runtime architecture session note", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                TYPED_ERROR_SOURCE_INVENTORY_METADATA_SPLIT,
                TYPED_ERROR_SOURCE_INVENTORY_METADATA_STATUS,
                TYPED_ERROR_SOURCE_INVENTORY_METADATA_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_METADATA_ROOT_PATHS_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_METADATA_CHILD_INVENTORY_PATHS_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_METADATA_DELEGATION_PATHS_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_METADATA_STATUS_SLICES_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_METADATA_REVIEW_GUARD_PATHS_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_METADATA_STATUS_CURRENT_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_METADATA_GUARD,
                TYPED_ERROR_SOURCE_INVENTORY_METADATA_STATUS_CURRENT_SPLIT,
                TYPED_ERROR_SOURCE_INVENTORY_METADATA_STATUS_CURRENT_STATUS,
                TYPED_ERROR_SOURCE_INVENTORY_METADATA_STATUS_CURRENT_ROUTE_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_METADATA_STATUS_CURRENT_MIRRORS_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_METADATA_STATUS_CURRENT_BUDGETS_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_METADATA_STATUS_CURRENT_SOURCE_BLOBS_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_METADATA_STATUS_CURRENT_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records typed-error source inventory metadata status-current split",
        &status_map,
        &[
            TYPED_ERROR_SOURCE_INVENTORY_METADATA_SPLIT,
            TYPED_ERROR_SOURCE_INVENTORY_METADATA_STATUS,
            TYPED_ERROR_SOURCE_INVENTORY_METADATA_STATUS_CURRENT_SPLIT,
            TYPED_ERROR_SOURCE_INVENTORY_METADATA_STATUS_CURRENT_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records typed-error source inventory metadata status-current split",
        &date_map,
        &[
            TYPED_ERROR_SOURCE_INVENTORY_METADATA_SPLIT,
            TYPED_ERROR_SOURCE_INVENTORY_METADATA_DATE,
            TYPED_ERROR_SOURCE_INVENTORY_METADATA_STATUS_CURRENT_SPLIT,
            TYPED_ERROR_SOURCE_INVENTORY_METADATA_STATUS_CURRENT_DATE,
        ],
    );
}
