use super::super::super::super::*;
use super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_status_docs_status_anchors_are_folder_backed() {
    let status_rows = review_guard_status_rows_source();
    let status_map = review_guard_status_map_source();
    let date_map = review_guard_date_map_source();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let required = [
        status_anchors::STATUS_DOC_STATUS_ANCHORS_FOLDER_BACKED_SPLIT_NAME,
        status_anchors::STATUS_DOC_STATUS_ANCHORS_FOLDER_BACKED_SPLIT_ID,
        STATUS_DOC_STATUS_ANCHORS_OWNER,
        status_anchors::STATUS_DOC_STATUS_CHILD_ANCHORS_OWNER,
        status_anchors::STATUS_DOC_STATUS_MAP_ANCHORS_OWNER,
        "runtime_15_code_review_findings_status_docs_status_child_anchors_are_child_owned",
        "runtime_15_code_review_findings_status_docs_status_map_anchors_are_child_owned",
        "runtime_15_code_review_findings_status_docs_status_anchors_are_folder_backed",
        "Cargo gate deferred",
    ];

    assert_contains_all(
        "review-guard status map",
        &status_map,
        &[
            status_anchors::STATUS_DOC_STATUS_ANCHORS_FOLDER_BACKED_SPLIT_NAME,
            status_anchors::STATUS_DOC_STATUS_ANCHORS_FOLDER_BACKED_SPLIT_ID,
        ],
    );
    assert_contains_all(
        "review-guard date map",
        &date_map,
        &[
            status_anchors::STATUS_DOC_STATUS_ANCHORS_FOLDER_BACKED_SPLIT_NAME,
            "2026-07-04",
        ],
    );
    for (label, source) in [
        ("status row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("runtime implementation session", session_note.as_str()),
    ] {
        assert_contains_all(label, source, &required);
    }
}

#[test]
fn runtime_15_code_review_findings_status_docs_status_anchor_guard_folder_backed_status_is_current()
{
    let status_rows = review_guard_status_rows_source();
    let status_map = review_guard_status_map_source();
    let date_map = review_guard_date_map_source();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let required = [
        STATUS_DOC_STATUS_ANCHOR_GUARD_FOLDER_BACKED_SLICE,
        STATUS_DOC_STATUS_ANCHOR_GUARD_FOLDER_BACKED_STATUS,
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status/status_anchor_guard.rs",
        STATUS_DOC_STATUS_ANCHOR_GUARD_CHILD_OWNERSHIP_CHILD,
        STATUS_DOC_STATUS_ANCHOR_GUARD_FOLDER_BACKING_CHILD,
        STATUS_DOC_STATUS_ANCHOR_GUARD_BUDGETS_CHILD,
        STATUS_DOC_STATUS_ANCHOR_GUARD_STATUS_MIRRORS_CHILD,
        "runtime_15_code_review_findings_status_docs_status_anchors_are_child_owner",
        STATUS_DOC_STATUS_ANCHOR_GUARD_FOLDER_BACKED_GUARD,
        STATUS_DOC_STATUS_ANCHOR_GUARD_STATUS_GUARD,
        STATUS_DOC_STATUS_ANCHOR_GUARD_BUDGET_GUARD,
        "Cargo gate deferred",
    ];

    for (label, source) in [
        ("status row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("runtime implementation session", session_note.as_str()),
    ] {
        assert_contains_all(label, source, &required);
    }
    assert_contains_all(
        "review-guard status map records status-anchor guard folder-backed split",
        &status_map,
        &[
            STATUS_DOC_STATUS_ANCHOR_GUARD_FOLDER_BACKED_SLICE,
            STATUS_DOC_STATUS_ANCHOR_GUARD_FOLDER_BACKED_STATUS,
        ],
    );
    assert_contains_all(
        "review-guard date map records status-anchor guard folder-backed split",
        &date_map,
        &[
            STATUS_DOC_STATUS_ANCHOR_GUARD_FOLDER_BACKED_SLICE,
            STATUS_DOC_STATUS_ANCHOR_GUARD_FOLDER_BACKED_DATE,
        ],
    );
}
