use super::*;

#[test]
fn runtime_15_review_guard_status_support_parent_has_no_anchor_mirror() {
    let parent = read_runtime_src(REVIEW_GUARD_STATUS_SUPPORT_ROWS_PARENT_PATH);
    let review_guard_rows = read_runtime_src(REVIEW_GUARD_STATUS_SUPPORT_ROWS_PATH);
    let typed_error_status_doc_rows =
        read_runtime_src(REVIEW_GUARD_STATUS_SUPPORT_TYPED_ERROR_STATUS_DOC_ROWS_PATH);
    let source_inventory_foundation_rows =
        read_runtime_src(REVIEW_GUARD_STATUS_SUPPORT_SOURCE_INVENTORY_FOUNDATION_ROWS_PATH);
    let source_inventory_inventory_metadata_rows =
        read_runtime_src(REVIEW_GUARD_STATUS_SUPPORT_SOURCE_INVENTORY_INVENTORY_METADATA_ROWS_PATH);
    let source_inventory_delegation_rows =
        read_runtime_src(REVIEW_GUARD_STATUS_SUPPORT_SOURCE_INVENTORY_DELEGATION_ROWS_PATH);
    let child_rows = [
        review_guard_rows.as_str(),
        typed_error_status_doc_rows.as_str(),
        source_inventory_foundation_rows.as_str(),
        source_inventory_inventory_metadata_rows.as_str(),
        source_inventory_delegation_rows.as_str(),
    ]
    .join("\n");

    for forbidden in [
        "Anchor mirror for older structure guards",
        "runtime_15_typed_error_source_inventory_guard_folder_backed_static_passed_cargo_deferred",
        "runtime_15_review_guard_direct_assertion_row_data_child_owner_split_static_passed_cargo_deferred",
        "runtime_15_review_guard_typed_error_row_data_child_split_static_passed_cargo_deferred",
    ] {
        assert!(
            !parent.contains(forbidden),
            "status_support_rows.rs should delegate historical anchor mirrors to child row owners; found {forbidden}"
        );
    }

    assert_contains_all(
        "status-support child rows retain representative historical anchors",
        &child_rows,
        &[
            "Runtime 15 M3 typed-error source inventory guard folder-backed split",
            "Runtime 15 M3 typed-error source inventory delegation folder-backed ownership child split",
            "Runtime 15 M3 review-guard direct-assertion row-data child-owner split",
            "Runtime 15 M3 review-guard typed-error row-data child split",
        ],
    );
}
