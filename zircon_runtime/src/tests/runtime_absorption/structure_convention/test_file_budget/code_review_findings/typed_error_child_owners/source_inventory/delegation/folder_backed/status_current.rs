use super::super::super::super::super::super::*;
use super::super::super::*;

#[test]
fn runtime_15_typed_error_source_inventory_delegation_folder_backed_status_is_current() {
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
                TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_SPLIT,
                TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_STATUS,
                TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_CHILD_OWNERSHIP_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_GUARD_BODY_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_STATUS_CHILD,
                TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_OWNERSHIP_GUARD,
                TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_STATUS_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records typed-error source inventory delegation folder-backed split",
        &status_map,
        &[
            TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_SPLIT,
            TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records typed-error source inventory delegation folder-backed split",
        &date_map,
        &[
            TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_SPLIT,
            TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_FOLDER_BACKED_DATE,
        ],
    );
}
