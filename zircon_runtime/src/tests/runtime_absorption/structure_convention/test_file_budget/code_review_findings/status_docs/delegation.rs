use super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_status_docs_folder_backed_status_is_current() {
    let parent = read_runtime_src(STATUS_DOC_PARENT_PATH);
    let child_sources = status_doc_child_source_blob();

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
            "#[path = \"status_docs/status_mirrors.rs\"]",
            "mod status_mirrors;",
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
