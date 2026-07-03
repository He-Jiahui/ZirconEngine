use super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_status_docs_folder_backed_status_is_current() {
    let parent = read_runtime_src(STATUS_DOC_PARENT_PATH);
    let child_sources = status_doc_child_source_blob();
    let status_rows = review_guard_status_rows_source();
    let status_map = read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH);
    let date_map = read_runtime_src(REVIEW_GUARD_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    assert_contains_all(
        "status-doc parent mounts folder-backed children",
        &parent,
        &[
            "#[path = \"status_docs/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"status_docs/source_anchor_guard.rs\"]",
            "mod source_anchor_guard;",
            "#[path = \"status_docs/status_anchor_guard.rs\"]",
            "mod status_anchor_guard;",
            "#[path = \"status_docs/sync.rs\"]",
            "mod sync;",
            STATUS_DOC_FOLDER_BACKED_SPLIT_NAME,
            STATUS_DOC_FOLDER_BACKED_SPLIT_ID,
        ],
    );
    for (_, child_path, guard_name) in STATUS_DOC_CHILDREN {
        assert!(
            parent.contains(child_path),
            "status-doc parent should inventory child path {child_path}"
        );
        assert!(
            child_sources.contains(guard_name),
            "status-doc child {child_path} should define {guard_name}"
        );
    }
    for moved_anchor in [
        "let runtime_15_plan =",
        "let status_rows = review_guard_status_rows_source();",
        "source_anchors::assert_code_review_findings_status_doc_source_anchors",
        "fn runtime_15_code_review_findings_status_docs_source_anchors_are_child_owner",
        "fn runtime_15_code_review_findings_status_docs_status_anchors_are_child_owner",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "status-doc implementation anchor `{moved_anchor}` should stay in folder-backed children"
        );
        assert!(
            child_sources.contains(moved_anchor),
            "status-doc children should own implementation anchor `{moved_anchor}`"
        );
    }
    for (label, source) in [
        ("structure guard row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("runtime implementation session", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                STATUS_DOC_FOLDER_BACKED_SPLIT_NAME,
                STATUS_DOC_FOLDER_BACKED_SPLIT_ID,
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs.rs",
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/sync.rs",
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/source_anchor_guard.rs",
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchor_guard.rs",
                "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/delegation.rs",
                "runtime_15_code_review_findings_status_docs_are_child_owner",
            ],
        );
    }
    assert_contains_all(
        "review-guard status map",
        &status_map,
        &[
            STATUS_DOC_FOLDER_BACKED_SPLIT_NAME,
            STATUS_DOC_FOLDER_BACKED_SPLIT_ID,
        ],
    );
    assert_contains_all(
        "review-guard date map",
        &date_map,
        &[STATUS_DOC_FOLDER_BACKED_SPLIT_NAME, "2026-07-02"],
    );

    for (path, source) in status_doc_child_sources()
        .into_iter()
        .chain([(STATUS_DOC_PARENT_PATH, parent)])
    {
        let line_count = source.lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the focused status-doc guard budget; got {line_count} lines"
        );
    }
}
