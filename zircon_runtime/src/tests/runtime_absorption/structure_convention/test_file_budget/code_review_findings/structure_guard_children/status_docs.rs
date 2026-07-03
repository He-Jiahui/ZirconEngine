use super::super::super::*;
use super::*;

pub(super) fn assert_code_review_status_doc_children_are_mounted() {
    let status_docs_child = read_runtime_src(STATUS_DOCS_CHILD_OWNER);
    let status_docs_sync_child = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/sync.rs",
    );
    let status_docs_source_anchor_guard_child = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/source_anchor_guard.rs",
    );
    let status_docs_status_anchor_guard_child = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/status_anchor_guard.rs",
    );
    let status_docs_delegation_child = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/delegation.rs",
    );
    let status_docs_source_anchors_child = read_runtime_src(STATUS_DOCS_SOURCE_ANCHORS_CHILD_OWNER);
    let status_docs_status_anchors_child = read_runtime_src(STATUS_DOCS_STATUS_ANCHORS_CHILD_OWNER);
    let status_docs_source_anchor_review_child = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/source_anchors/review_sources.rs",
    );
    let status_docs_source_anchor_native_child = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/source_anchors/native_typed_error.rs",
    );
    let status_docs_source_anchor_runtime_child = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/source_anchors/runtime_surface.rs",
    );
    let status_docs_source_anchor_structure_child = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/source_anchors/structure_owners.rs",
    );
    let status_docs_source_anchor_status_child = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs/source_anchors/status_mirrors.rs",
    );
    let status_docs_child_tree = [
        status_docs_child.as_str(),
        status_docs_sync_child.as_str(),
        status_docs_source_anchor_guard_child.as_str(),
        status_docs_status_anchor_guard_child.as_str(),
        status_docs_delegation_child.as_str(),
        status_docs_source_anchors_child.as_str(),
        status_docs_source_anchor_review_child.as_str(),
        status_docs_source_anchor_native_child.as_str(),
        status_docs_source_anchor_runtime_child.as_str(),
        status_docs_source_anchor_structure_child.as_str(),
        status_docs_source_anchor_status_child.as_str(),
        status_docs_status_anchors_child.as_str(),
    ]
    .join("\n");

    assert_contains_all(
        "code review findings status-doc structure child owner keeps status/document checks",
        &status_docs_child_tree,
        &[
            "fn runtime_15_code_review_findings_status_docs_are_child_owner",
            "#[path = \"status_docs/source_anchors.rs\"]",
            "mod source_anchors;",
            "#[path = \"status_docs/status_anchors.rs\"]",
            "mod status_anchors;",
            "source_anchors::assert_code_review_findings_status_doc_source_anchors",
            "status_anchors::STATUS_DOC_CHILD_ANCHORS",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status_docs.rs",
            "runtime_15_code_review_findings_status_docs_are_child_owner",
            "runtime_15_code_review_findings_status_docs_source_anchors_are_child_owner",
            "runtime_15_code_review_findings_status_docs_status_anchors_are_child_owner",
        ],
    );
    assert_contains_all(
        "code review findings status-doc anchor children keep status/document checks",
        &status_docs_child_tree,
        &[
            "Runtime 15 M3 code review findings status-doc guard child-owner split",
            "runtime_15_code_review_findings_status_docs_child_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 code review findings test folder split",
            "Runtime 15 M3 P0 native fixture review guard leaf-owner split",
            "Runtime 15 M3 code review findings structure guard child-owner split",
            "runtime_15_code_review_findings_tests_are_folder_backed",
            "runtime_15_code_review_findings_structure_guard_children_are_mounted",
        ],
    );
}

#[test]
fn runtime_15_code_review_findings_structure_guard_status_docs_are_child_owned() {
    assert_code_review_status_doc_children_are_mounted();
}
