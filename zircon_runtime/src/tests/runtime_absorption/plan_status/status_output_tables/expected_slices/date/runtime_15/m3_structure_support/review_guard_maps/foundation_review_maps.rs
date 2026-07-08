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

pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    if let Some(date) = expected_slice_rows::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = code_review_rows::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = p0_rows::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = f8_rows::expected_date_for_slice(slice) {
        return Some(date);
    }
    if let Some(date) = late_api_rows::expected_date_for_slice(slice) {
        return Some(date);
    }

    None
}
