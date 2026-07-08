use super::*;

#[test]
fn runtime_15_code_review_findings_status_docs_child_anchor_list_is_child_owned() {
    assert_status_doc_child_anchor_route_is_folder_backed();
}

#[test]
fn runtime_15_code_review_findings_status_docs_child_anchor_route_is_folder_backed() {
    assert_status_doc_child_anchor_route_is_folder_backed();
}

fn assert_status_doc_child_anchor_route_is_folder_backed() {
    let parent = read_runtime_src(STATUS_DOC_STATUS_CHILD_ANCHORS_OWNER);
    let child_blob = status_doc_child_anchor_child_source_blob();
    let anchors = status_doc_child_anchors();

    assert_contains_all(
        "status-doc child-anchor route mounts focused children",
        &parent,
        &[
            r#"#[path = "child_anchors/aggregation.rs"]"#,
            r#"#[path = "child_anchors/boundary_samples.rs"]"#,
            r#"#[path = "child_anchors/inventory.rs"]"#,
            r#"#[path = "child_anchors/source_blob.rs"]"#,
            r#"#[path = "child_anchors/split_layout.rs"]"#,
        ],
    );

    for moved_anchor in [
        "pub(super) const STATUS_DOC_CHILD_ANCHOR_LIST_SPLIT_NAME",
        "pub(super) const STATUS_DOC_CHILD_ANCHOR_CHILDREN",
        "pub(super) fn status_doc_child_anchor_child_source_blob",
        "fn status_doc_child_anchor_boundary_samples",
        "fn runtime_15_code_review_findings_status_docs_child_anchor_list_is_child_owned",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "child_anchors.rs should delegate route-owned `{moved_anchor}` to child modules"
        );
    }

    for (_, child_path) in STATUS_DOC_CHILD_ANCHOR_CHILDREN {
        assert!(
            parent.contains(child_path) || child_blob.contains(child_path),
            "status-doc child-anchor route should keep child path {child_path} reachable"
        );
    }
    for moved_anchor in boundary_samples::status_doc_child_anchor_boundary_samples() {
        assert!(
            !parent.contains(moved_anchor),
            "child_anchors.rs should delegate concrete child anchor `{moved_anchor}` to child files"
        );
        assert!(
            child_blob.contains(moved_anchor),
            "child-anchor child files should own concrete child anchor `{moved_anchor}`"
        );
        assert!(
            anchors.contains(&moved_anchor),
            "status_doc_child_anchors() should aggregate concrete child anchor `{moved_anchor}`"
        );
    }
    assert!(anchors
        .contains(&"runtime_15_code_review_findings_status_docs_status_anchors_are_child_owner"));
    assert!(anchors.contains(&STATUS_DOC_CHILD_ANCHOR_LIST_SPLIT_NAME));
    assert!(anchors.contains(&STATUS_DOC_CHILD_ANCHOR_LIST_SPLIT_ID));
    assert!(anchors.contains(&STATUS_DOC_CHILD_ANCHOR_LIST_SPLIT_GUARD));
    assert!(anchors.contains(&STATUS_DOC_CHILD_ANCHOR_ROUTE_FOLDER_BACKED_SPLIT_NAME));
    assert!(anchors.contains(&STATUS_DOC_CHILD_ANCHOR_ROUTE_FOLDER_BACKED_SPLIT_ID));
    assert!(anchors.contains(&STATUS_DOC_CHILD_ANCHOR_ROUTE_FOLDER_BACKED_SPLIT_GUARD));

    let row_data = status_doc_review_guard_status_rows_source();
    assert_contains_all(
        "status-doc child-anchor route split row data",
        &row_data,
        &[
            STATUS_DOC_CHILD_ANCHOR_ROUTE_FOLDER_BACKED_SPLIT_NAME,
            STATUS_DOC_CHILD_ANCHOR_ROUTE_FOLDER_BACKED_SPLIT_ID,
            STATUS_DOC_CHILD_ANCHOR_ROUTE_FOLDER_BACKED_SPLIT_GUARD,
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status/status_anchors/child_anchors/aggregation.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/status/status_anchors/child_anchors/split_layout.rs",
        ],
    );

    let status_map = status_doc_review_guard_status_map_source();
    assert_contains_all(
        "status-doc child-anchor route split status map",
        &status_map,
        &[
            STATUS_DOC_CHILD_ANCHOR_ROUTE_FOLDER_BACKED_SPLIT_NAME,
            STATUS_DOC_CHILD_ANCHOR_ROUTE_FOLDER_BACKED_SPLIT_ID,
        ],
    );

    let date_map = status_doc_review_guard_date_map_source();
    assert_contains_all(
        "status-doc child-anchor route split date map",
        &date_map,
        &[
            STATUS_DOC_CHILD_ANCHOR_ROUTE_FOLDER_BACKED_SPLIT_NAME,
            STATUS_DOC_CHILD_ANCHOR_ROUTE_FOLDER_BACKED_SPLIT_DATE,
        ],
    );
}

fn assert_contains_all(label: &str, source: &str, needles: &[&str]) {
    let missing = needles
        .iter()
        .copied()
        .filter(|needle| !source.contains(needle))
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "{label} missing expected anchors:\n{}",
        missing.join("\n")
    );
}
