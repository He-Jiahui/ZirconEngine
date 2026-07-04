use super::*;

pub(super) fn assert_status_doc_mounts_status_anchor_child_owner(parent: &str) {
    assert_contains_all(
        "status-doc guard mounts status-anchor child owner",
        parent,
        &[
            "#[path = \"status_docs/status_anchors.rs\"]",
            "mod status_anchors;",
            "#[path = \"status_docs/status_anchor_guard.rs\"]",
            "mod status_anchor_guard;",
        ],
    );
    assert!(
        !parent.contains("pub(super) const STATUS_DOC_STATUS_ANCHORS_OWNER"),
        "status-doc parent should not own the status-anchor owner path constant"
    );

    let root_paths = read_runtime_src(STATUS_DOC_ROOT_PATHS_CHILD);
    assert_contains_all(
        "status-doc root path child owns status-anchor owner path",
        &root_paths,
        &[
            "pub(super) const STATUS_DOC_STATUS_ANCHORS_OWNER",
            STATUS_DOC_STATUS_ANCHORS_OWNER,
        ],
    );
}

pub(super) fn assert_status_doc_children_own_status_anchor_assertions(child_tree: &str) {
    assert_contains_all(
        "status-doc guard children own status-anchor assertions",
        child_tree,
        &[
            "status_anchors::status_doc_child_anchors",
            "status_anchors::STATUS_DOC_MAP_ANCHORS",
            "status_anchors::status_doc_session_anchors",
            "fn runtime_15_code_review_findings_status_docs_status_anchors_are_child_owner",
        ],
    );
}
