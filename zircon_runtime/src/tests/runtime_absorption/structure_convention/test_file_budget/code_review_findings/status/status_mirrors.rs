use super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_status_docs_status_mirrors_are_child_owner() {
    let delegation = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status/delegation.rs",
    );
    let status_mirrors = read_runtime_src(STATUS_DOC_STATUS_MIRRORS_OWNER);

    for moved_anchor in [
        "let status_rows = review_guard_status_rows_source();",
        "let runtime_15_plan =",
        "review-guard status map",
        "review-guard date map",
    ] {
        assert!(
            !delegation.contains(moved_anchor),
            "status-doc delegation child should not own status mirror anchor `{moved_anchor}`"
        );
        assert!(
            status_mirrors.contains(moved_anchor),
            "status-doc status mirrors child should own status mirror anchor `{moved_anchor}`"
        );
    }

    assert_status_doc_status_mirrors_are_current();
}

fn assert_status_doc_status_mirrors_are_current() {
    let status_rows = review_guard_status_rows_source();
    let status_map = review_guard_status_map_source();
    let date_map = review_guard_date_map_source();
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
                STATUS_DOC_FOLDER_BACKED_SPLIT_NAME,
                STATUS_DOC_FOLDER_BACKED_SPLIT_ID,
                STATUS_DOC_STATUS_MIRROR_CHILD_OWNER_SLICE,
                STATUS_DOC_STATUS_MIRROR_CHILD_OWNER_STATUS,
                STATUS_DOC_MAP_SOURCE_SYNC_SLICE,
                STATUS_DOC_MAP_SOURCE_SYNC_STATUS,
                STATUS_DOC_PARENT_PATH,
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status/sync.rs",
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status/source_anchor_guard.rs",
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status/status_anchor_guard.rs",
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status/delegation.rs",
                STATUS_DOC_STATUS_MIRRORS_OWNER,
                "runtime_15_code_review_findings_status_docs_are_child_owner",
                STATUS_DOC_STATUS_MIRROR_CHILD_OWNER_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "review-guard status map",
        &status_map,
        &[
            STATUS_DOC_FOLDER_BACKED_SPLIT_NAME,
            STATUS_DOC_FOLDER_BACKED_SPLIT_ID,
            STATUS_DOC_STATUS_MIRROR_CHILD_OWNER_SLICE,
            STATUS_DOC_STATUS_MIRROR_CHILD_OWNER_STATUS,
            STATUS_DOC_MAP_SOURCE_SYNC_SLICE,
            STATUS_DOC_MAP_SOURCE_SYNC_STATUS,
        ],
    );
    assert_contains_all(
        "review-guard date map",
        &date_map,
        &[
            STATUS_DOC_FOLDER_BACKED_SPLIT_NAME,
            "2026-07-02",
            STATUS_DOC_STATUS_MIRROR_CHILD_OWNER_SLICE,
            STATUS_DOC_STATUS_MIRROR_CHILD_OWNER_DATE,
            STATUS_DOC_MAP_SOURCE_SYNC_SLICE,
            STATUS_DOC_MAP_SOURCE_SYNC_DATE,
        ],
    );
}
