use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_typed_error_status_doc_source_helper_status_is_current() {
    let status_rows = typed_error_status_row_source();
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
        ("typed-error row data", status_rows.as_str()),
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
                TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_SPLIT_NAME,
                TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_SPLIT_ID,
                TYPED_ERROR_STATUS_DOCS_CHILD,
                TYPED_ERROR_STATUS_DOCS_CHILD_SOURCES_CHILD,
                TYPED_ERROR_STATUS_DOCS_PATHS_CHILD,
                TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_OWNERSHIP_CHILD,
                TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_STATUS_CHILD,
                TYPED_ERROR_STATUS_DOCS_SOURCES_CHILD,
                TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_OWNERSHIP_GUARD,
                TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_STATUS_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records typed-error status-doc source helper split",
        &status_map,
        &[
            TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_SPLIT_NAME,
            TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_SPLIT_ID,
        ],
    );
    assert_contains_all(
        "M3 review date map records typed-error status-doc source helper split",
        &date_map,
        &[
            TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_SPLIT_NAME,
            TYPED_ERROR_STATUS_DOCS_SOURCE_HELPER_SPLIT_DATE,
        ],
    );
}
