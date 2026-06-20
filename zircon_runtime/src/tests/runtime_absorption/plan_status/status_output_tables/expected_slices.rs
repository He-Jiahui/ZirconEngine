#[path = "expected_slices/date.rs"]
mod date;
#[path = "expected_slices/status.rs"]
mod status;

pub(super) fn expected_status_for_slice(slice: &str) -> &'static str {
    status::expected_status_for_slice(slice)
}

pub(super) fn expected_date_for_slice(slice: &str) -> &'static str {
    date::expected_date_for_slice(slice)
}
