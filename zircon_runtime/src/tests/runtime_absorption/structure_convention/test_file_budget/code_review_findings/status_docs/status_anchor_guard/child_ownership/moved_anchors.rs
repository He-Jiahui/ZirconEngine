use super::*;

const MOVED_STATUS_ANCHORS: &[&str] = &[
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
];

const REQUIRED_STATUS_CHILD_ANCHORS: &[&str] = &[
    "runtime_15_code_review_findings_status_docs_status_anchors_are_child_owner",
    "runtime_15_code_review_findings_status_docs_status_child_anchors_are_child_owned",
    "runtime_15_code_review_findings_status_docs_status_map_anchors_are_child_owned",
];

pub(super) fn assert_moved_status_anchors_stay_child_owned(
    parent: &str,
    child: &str,
    status_anchor_child_tree: &str,
) {
    for moved_anchor in MOVED_STATUS_ANCHORS {
        assert!(
            !parent.contains(moved_anchor),
            "status_docs.rs should delegate status anchor `{moved_anchor}` to status_docs/status_anchors.rs"
        );
        assert!(
            !child.contains(moved_anchor),
            "status_anchors.rs should delegate status anchor `{moved_anchor}` to status_anchors children"
        );
        assert!(
            status_anchor_child_tree.contains(moved_anchor),
            "status_anchors children should own moved status anchor `{moved_anchor}`"
        );
    }
}

pub(super) fn assert_status_anchor_children_own_required_status_anchors(
    status_anchor_child_tree: &str,
) {
    assert_contains_all(
        "status-anchor children own status-doc slice/status/owner/guard anchors",
        status_anchor_child_tree,
        &[MOVED_STATUS_ANCHORS, REQUIRED_STATUS_CHILD_ANCHORS].concat(),
    );
}
