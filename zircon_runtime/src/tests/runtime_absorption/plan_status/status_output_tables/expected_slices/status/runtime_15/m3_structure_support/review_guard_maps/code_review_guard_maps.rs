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

pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if let Some(status) = expected_slice_rows::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = structure_guard_rows::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = status_doc_rows::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = folder_backed_summary_rows::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = source_inventory_rows::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = direct_assertion_rows::expected_status_for_slice(slice) {
        return Some(status);
    }

    None
}
