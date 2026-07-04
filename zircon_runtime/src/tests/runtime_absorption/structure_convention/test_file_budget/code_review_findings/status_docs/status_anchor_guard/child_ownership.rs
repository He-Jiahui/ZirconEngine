use super::super::super::super::*;
use super::super::*;
use super::*;

#[path = "child_ownership/budgets.rs"]
mod budgets;
#[path = "child_ownership/forwarding.rs"]
mod forwarding;
#[path = "child_ownership/mounts.rs"]
mod mounts;
#[path = "child_ownership/moved_anchors.rs"]
mod moved_anchors;
#[path = "child_ownership/status_mirrors.rs"]
mod status_mirrors;

const STATUS_DOC_STATUS_ANCHOR_CHILD_OWNERSHIP_SPLIT_NAME: &str =
    "Runtime 15 M3 code review findings status-doc status-anchor child-ownership child split";
const STATUS_DOC_STATUS_ANCHOR_CHILD_OWNERSHIP_SPLIT_ID: &str =
    "runtime_15_code_review_findings_status_docs_status_anchor_child_ownership_child_split_static_passed_cargo_deferred";
const STATUS_DOC_STATUS_ANCHOR_CHILD_OWNERSHIP_SPLIT_DATE: &str = "2026-07-05";
const STATUS_DOC_STATUS_ANCHOR_CHILD_OWNERSHIP_SPLIT_GUARD: &str =
    "runtime_15_code_review_findings_status_docs_status_anchor_child_ownership_is_child_backed";

const STATUS_DOC_STATUS_ANCHOR_CHILD_OWNERSHIP_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "mounts",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchor_guard/child_ownership/mounts.rs",
        "assert_status_doc_mounts_status_anchor_child_owner",
    ),
    (
        "moved_anchors",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchor_guard/child_ownership/moved_anchors.rs",
        "assert_moved_status_anchors_stay_child_owned",
    ),
    (
        "forwarding",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchor_guard/child_ownership/forwarding.rs",
        "assert_status_anchor_route_forwards_children",
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchor_guard/child_ownership/budgets.rs",
        "assert_status_anchor_line_budgets",
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchor_guard/child_ownership/status_mirrors.rs",
        STATUS_DOC_STATUS_ANCHOR_CHILD_OWNERSHIP_SPLIT_GUARD,
    ),
];

#[test]
fn runtime_15_code_review_findings_status_docs_status_anchors_are_child_owner() {
    let parent = read_runtime_src(STATUS_DOC_PARENT_PATH);
    let child = read_runtime_src(STATUS_DOC_STATUS_ANCHORS_OWNER);
    let child_tree = status_doc_child_source_blob();
    let status_anchor_child_tree = status_anchors::status_doc_status_anchor_child_source_blob();

    mounts::assert_status_doc_mounts_status_anchor_child_owner(&parent);
    mounts::assert_status_doc_children_own_status_anchor_assertions(&child_tree);
    moved_anchors::assert_moved_status_anchors_stay_child_owned(
        &parent,
        &child,
        &status_anchor_child_tree,
    );
    forwarding::assert_status_anchor_route_forwards_children(&child);
    forwarding::assert_retired_status_anchor_consts_are_absent(&child);
    moved_anchors::assert_status_anchor_children_own_required_status_anchors(
        &status_anchor_child_tree,
    );
    assert_code_review_findings_status_docs_are_synced();
    budgets::assert_status_anchor_line_budgets(&parent, &child);
}
