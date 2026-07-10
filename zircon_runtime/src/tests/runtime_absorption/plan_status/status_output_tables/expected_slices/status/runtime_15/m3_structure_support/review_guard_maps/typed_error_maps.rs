#[path = "typed_error_maps/expected_slice_rows.rs"]
mod expected_slice_rows;
#[path = "typed_error_maps/review_guard_rows.rs"]
mod review_guard_rows;
#[path = "typed_error_maps/row_data_rows.rs"]
mod row_data_rows;
#[path = "typed_error_maps/source_inventory_rows.rs"]
mod source_inventory_rows;
#[path = "typed_error_maps/status_doc_rows.rs"]
mod status_doc_rows;

pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review-guard typed-error rows guard folder-backed split" => {
            return Some(
                "runtime_15_review_guard_typed_error_rows_guard_folder_backed_static_passed_cargo_deferred",
            );
        }
        "Runtime 15 M3 review-guard typed-error rows split-layout guard folder-backed split" => {
            return Some(
                "runtime_15_review_guard_typed_error_rows_split_layout_guard_folder_backed_static_passed_cargo_deferred",
            );
        }
        "Runtime 15 M3 review-guard status-support row-data folder-backed split" => {
            return Some(
                "runtime_15_review_guard_status_support_row_data_folder_backed_static_passed_cargo_deferred",
            );
        }
        "Runtime 15 M3 review-guard status-support rows guard folder-backed split" => {
            return Some(
                "runtime_15_review_guard_status_support_rows_guard_folder_backed_static_passed_cargo_deferred",
            );
        }
        "Runtime 15 M3 review-guard status-support folder-backed guard folder-backed split" => {
            return Some(
                "runtime_15_review_guard_status_support_folder_backed_guard_folder_backed_static_passed_cargo_deferred",
            );
        }
        "Runtime 15 M3 review-guard status-support rows split-layout guard folder-backed split" => {
            return Some(
                "runtime_15_review_guard_status_support_rows_split_layout_guard_folder_backed_static_passed_cargo_deferred",
            );
        }
        "Runtime 15 M3 review-guard status-support anchor mirror cleanup" => {
            return Some(
                "runtime_15_review_guard_status_support_anchor_mirror_cleanup_static_passed_cargo_deferred",
            );
        }
        "Runtime 15 M3 review-guard status-support folder-backed split-layout guard folder-backed split" => {
            return Some(
                "runtime_15_review_guard_status_support_folder_backed_split_layout_guard_folder_backed_static_passed_cargo_deferred",
            );
        }
        "Runtime 15 M3 review-guard status-support anchor-mirror cleanup guard folder-backed split" => {
            return Some(
                "runtime_15_review_guard_status_support_anchor_mirror_cleanup_guard_folder_backed_static_passed_cargo_deferred",
            );
        }
        _ => {}
    }
    if let Some(status) = expected_slice_rows::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = review_guard_rows::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = row_data_rows::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = status_doc_rows::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = source_inventory_rows::expected_status_for_slice(slice) {
        return Some(status);
    }

    None
}
