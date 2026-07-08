#[path = "foundation_review_maps/code_review_rows.rs"]
mod code_review_rows;
#[path = "foundation_review_maps/expected_slice_rows.rs"]
mod expected_slice_rows;
#[path = "foundation_review_maps/f8_rows.rs"]
mod f8_rows;
#[path = "foundation_review_maps/late_api_rows.rs"]
mod late_api_rows;
#[path = "foundation_review_maps/p0_rows.rs"]
mod p0_rows;

pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if let Some(status) = expected_slice_rows::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = code_review_rows::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = p0_rows::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = f8_rows::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = late_api_rows::expected_status_for_slice(slice) {
        return Some(status);
    }

    None
}
