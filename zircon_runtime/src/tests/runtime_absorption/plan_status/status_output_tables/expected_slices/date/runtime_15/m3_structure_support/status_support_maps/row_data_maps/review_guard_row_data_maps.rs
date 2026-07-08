#[path = "review_guard_row_data_maps/base_child_owner_maps.rs"]
mod base_child_owner_maps;
#[path = "review_guard_row_data_maps/code_review_maps.rs"]
mod code_review_maps;
#[path = "review_guard_row_data_maps/direct_assertion_maps.rs"]
mod direct_assertion_maps;
#[path = "review_guard_row_data_maps/moved_row_maps.rs"]
mod moved_row_maps;
#[path = "review_guard_row_data_maps/row_data_guard_maps.rs"]
mod row_data_guard_maps;
#[path = "review_guard_row_data_maps/status_doc_maps.rs"]
mod status_doc_maps;

const AGGREGATION_CHILD_SPLIT_STATUS_NAME: &str =
    "Runtime 15 M3 review-guard row-data aggregation guard child split";
const AGGREGATION_CHILD_SPLIT_DATE: &str = "2026-07-05";

pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if slice == AGGREGATION_CHILD_SPLIT_STATUS_NAME {
        return Some(AGGREGATION_CHILD_SPLIT_DATE);
    }
    if let Some(date) = base_child_owner_maps::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = moved_row_maps::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = code_review_maps::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = row_data_guard_maps::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = status_doc_maps::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = direct_assertion_maps::expected_date_for_slice(slice) {
        return Some(date);
    }

    None
}
