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

pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 review-guard status-support row-data folder-backed split" => {
            return Some("2026-07-05");
        }
        "Runtime 15 M3 review-guard status-support rows guard folder-backed split"
        | "Runtime 15 M3 review-guard status-support folder-backed guard folder-backed split"
        | "Runtime 15 M3 review-guard status-support rows split-layout guard folder-backed split"
        | "Runtime 15 M3 review-guard status-support anchor mirror cleanup"
        | "Runtime 15 M3 review-guard status-support folder-backed split-layout guard folder-backed split"
        | "Runtime 15 M3 review-guard status-support anchor-mirror cleanup guard folder-backed split" => {
            return Some("2026-07-06");
        }
        _ => {}
    }
    if let Some(date) = expected_slice_rows::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = review_guard_rows::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = row_data_rows::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = status_doc_rows::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = source_inventory_rows::expected_date_for_slice(slice) {
        return Some(date);
    }

    None
}
