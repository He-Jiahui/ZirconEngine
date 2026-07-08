pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review-guard row-data status-doc guard folder-backed split" => Some(
            "runtime_15_review_guard_row_data_status_docs_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data status-doc status-mirror child split" => Some(
            "runtime_15_review_guard_row_data_status_docs_status_mirror_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data status-doc root inventory child split" => Some(
            "runtime_15_review_guard_row_data_status_docs_root_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard row-data status-doc source reconciliation" => Some(
            "runtime_15_review_guard_row_data_status_doc_source_reconciliation_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard typed-error status-doc row-data folder-backed split" => Some(
            "runtime_15_review_guard_typed_error_status_doc_row_data_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 review-guard typed-error status-doc guard folder-backed split" => Some(
            "runtime_15_review_guard_typed_error_status_doc_guard_folder_backed_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
