use super::super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_typed_error_status_doc_mirrors_folder_backed_status_is_current() {
    let status_rows = typed_error_status_row_source();
    let status_map = typed_error_status_map_source();
    let date_map = typed_error_date_map_source();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (label, source) in [
        ("typed-error row data", status_rows.as_str()),
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
                TYPED_ERROR_STATUS_DOC_MIRRORS_FOLDER_BACKED_SLICE,
                TYPED_ERROR_STATUS_DOC_MIRRORS_FOLDER_BACKED_STATUS,
                TYPED_ERROR_STATUS_DOC_MIRRORS_CHILD,
                TYPED_ERROR_STATUS_DOC_MIRRORS_STATUS_SLICES_CHILD,
                TYPED_ERROR_STATUS_DOC_MIRRORS_SOURCE_PATHS_CHILD,
                TYPED_ERROR_STATUS_DOC_MIRRORS_GUARD_ANCHORS_CHILD,
                TYPED_ERROR_STATUS_DOC_MIRRORS_STATUS_CURRENT_CHILD,
                "assert_typed_error_status_doc_mirrors_are_synced",
                TYPED_ERROR_STATUS_DOC_MIRRORS_FOLDER_BACKED_GUARD,
                TYPED_ERROR_STATUS_DOC_MIRRORS_STATUS_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records typed-error doc mirror folder-backed split",
        &status_map,
        &[
            TYPED_ERROR_STATUS_DOC_MIRRORS_FOLDER_BACKED_SLICE,
            TYPED_ERROR_STATUS_DOC_MIRRORS_FOLDER_BACKED_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records typed-error doc mirror folder-backed split",
        &date_map,
        &[
            TYPED_ERROR_STATUS_DOC_MIRRORS_FOLDER_BACKED_SLICE,
            TYPED_ERROR_STATUS_DOC_MIRRORS_FOLDER_BACKED_DATE,
        ],
    );
}
