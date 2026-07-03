use super::super::super::*;
use super::*;

#[test]
fn runtime_15_code_review_findings_status_docs_status_anchors_are_child_owner() {
    let parent = read_runtime_src(STATUS_DOC_PARENT_PATH);
    let child = read_runtime_src(STATUS_DOC_STATUS_ANCHORS_OWNER);
    let child_tree = status_doc_child_source_blob();

    assert_contains_all(
        "status-doc guard mounts status-anchor child owner",
        &parent,
        &[
            "#[path = \"status_docs/status_anchors.rs\"]",
            "mod status_anchors;",
            "#[path = \"status_docs/status_anchor_guard.rs\"]",
            "mod status_anchor_guard;",
            STATUS_DOC_STATUS_ANCHORS_OWNER,
        ],
    );
    assert_contains_all(
        "status-doc guard children own status-anchor assertions",
        &child_tree,
        &[
            "status_anchors::STATUS_DOC_CHILD_ANCHORS",
            "status_anchors::STATUS_DOC_MAP_ANCHORS",
            "status_anchors::STATUS_DOC_SESSION_ANCHORS",
            "fn runtime_15_code_review_findings_status_docs_status_anchors_are_child_owner",
        ],
    );
    for moved_anchor in [
        concat!(
            "Runtime 15 M3 code review findings F12 direct assertions ",
            "child-owner split"
        ),
        concat!(
            "runtime_15_code_review_findings_f12_direct_assertions_",
            "child_owner_split_static_passed_cargo_deferred"
        ),
        concat!(
            "tests/runtime_absorption/structure_convention/test_file_budget/",
            "code_review_findings/folder_backed_summary/direct_review_assertions/f12.rs"
        ),
        concat!(
            "runtime_15_plugin_importer_d13_sdk_structure_assertions_",
            "guard_child_owner_split_static_passed_cargo_deferred"
        ),
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "status_docs.rs should delegate status anchor `{moved_anchor}` to status_docs/status_anchors.rs"
        );
    }
    assert_contains_all(
        "status-anchor child owns status-doc slice/status/owner/guard anchors",
        &child,
        &[
            "pub(super) const STATUS_DOC_CHILD_ANCHORS",
            "pub(super) const STATUS_DOC_MAP_ANCHORS",
            "pub(super) const STATUS_DOC_SESSION_ANCHORS",
            concat!(
                "Runtime 15 M3 code review findings F12 direct assertions ",
                "child-owner split"
            ),
            concat!(
                "runtime_15_code_review_findings_f12_direct_assertions_",
                "child_owner_split_static_passed_cargo_deferred"
            ),
            concat!(
                "tests/runtime_absorption/structure_convention/test_file_budget/",
                "code_review_findings/folder_backed_summary/direct_review_assertions/f12.rs"
            ),
            concat!(
                "runtime_15_plugin_importer_d13_sdk_structure_assertions_",
                "guard_child_owner_split_static_passed_cargo_deferred"
            ),
            "runtime_15_code_review_findings_status_docs_status_anchors_are_child_owner",
        ],
    );
    assert_code_review_findings_status_docs_are_synced();

    for (path, source) in [
        (STATUS_DOC_PARENT_PATH, parent.as_str()),
        (STATUS_DOC_STATUS_ANCHORS_OWNER, child.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
