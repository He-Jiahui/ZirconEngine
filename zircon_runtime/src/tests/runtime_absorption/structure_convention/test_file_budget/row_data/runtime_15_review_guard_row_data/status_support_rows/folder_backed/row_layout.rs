use super::*;

#[test]
fn runtime_15_review_guard_status_support_rows_are_folder_backed() {
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

    assert_contains_all(
        "review-guard status-support row-data parent mounts focused children",
        &parent,
        &[
            "#[path = \"status_support_rows/review_guard_rows.rs\"]",
            "#[path = \"status_support_rows/typed_error_status_doc_rows.rs\"]",
            "#[path = \"status_support_rows/source_inventory_foundation_rows.rs\"]",
            "#[path = \"status_support_rows/source_inventory_inventory_metadata_rows.rs\"]",
            "#[path = \"status_support_rows/source_inventory_delegation_rows.rs\"]",
            "review_guard_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "typed_error_status_doc_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "source_inventory_foundation_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "source_inventory_inventory_metadata_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "source_inventory_delegation_rows::EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert!(
        !parent.contains(
            "pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &["
        ),
        "status_support_rows.rs should delegate row tuples to folder-backed children"
    );
    assert_contains_all(
        "status-support children retain representative row topics",
        &[
            review_guard_rows.as_str(),
            typed_error_status_doc_rows.as_str(),
            source_inventory_foundation_rows.as_str(),
            source_inventory_inventory_metadata_rows.as_str(),
            source_inventory_delegation_rows.as_str(),
        ]
        .join("\n"),
        &[
            "Runtime 15 M3 review guard row-data topic child-owner split",
            "Runtime 15 M3 typed-error structure status-doc guard child-owner split",
            "Runtime 15 M3 typed-error source inventory guard folder-backed split",
            "Runtime 15 M3 typed-error source inventory child inventory folder-backed split",
            "Runtime 15 M3 typed-error source inventory delegation folder-backed ownership child split",
        ],
    );
}
