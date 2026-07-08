#[path = "code_review_guard_maps/direct_assertion_rows.rs"]
mod direct_assertion_rows;
#[path = "code_review_guard_maps/expected_slice_rows.rs"]
mod expected_slice_rows;
#[path = "code_review_guard_maps/folder_backed_summary_rows.rs"]
mod folder_backed_summary_rows;
#[path = "code_review_guard_maps/source_inventory_rows.rs"]
mod source_inventory_rows;
#[path = "code_review_guard_maps/status_doc_rows.rs"]
mod status_doc_rows;
#[path = "code_review_guard_maps/structure_guard_rows.rs"]
mod structure_guard_rows;

pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if let Some(date) = expected_slice_rows::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = structure_guard_rows::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = status_doc_rows::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = folder_backed_summary_rows::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = source_inventory_rows::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = direct_assertion_rows::expected_date_for_slice(slice) {
        return Some(date);
    }

    None
}
