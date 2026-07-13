use super::super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_status_docs_source_anchors_folder_backed_status_is_current() {
    let status_rows = super::super::review_guard_status_rows_source();
    let status_map = super::super::review_guard_status_map_source();
    let date_map = super::super::review_guard_date_map_source();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (label, source) in [
        ("structure guard row data", status_rows.as_str()),
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
                SOURCE_ANCHORS_FOLDER_BACKED_SPLIT_NAME,
                SOURCE_ANCHORS_FOLDER_BACKED_SPLIT_ID,
                super::super::STATUS_DOC_SOURCE_ANCHORS_OWNER,
                SOURCE_ANCHORS_REVIEW_SOURCES_OWNER,
                SOURCE_ANCHORS_NATIVE_TYPED_ERROR_OWNER,
                SOURCE_ANCHORS_RUNTIME_SURFACE_OWNER,
                SOURCE_ANCHORS_STRUCTURE_OWNERS_OWNER,
                SOURCE_ANCHORS_STATUS_MIRRORS_OWNER,
                "runtime_15_code_review_findings_status_docs_source_anchors_are_child_owner",
                "runtime_15_code_review_findings_status_docs_source_anchors_folder_backed_status_is_current",
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "M3 review status map records status-doc source anchors folder-backed split",
        &status_map,
        &[
            SOURCE_ANCHORS_FOLDER_BACKED_SPLIT_NAME,
            SOURCE_ANCHORS_FOLDER_BACKED_SPLIT_ID,
        ],
    );
    assert_contains_all(
        "M3 review date map records status-doc source anchors folder-backed split",
        &date_map,
        &[SOURCE_ANCHORS_FOLDER_BACKED_SPLIT_NAME, "2026-07-02"],
    );

    assert_status_doc_source_anchor_children_are_mounted();
}
