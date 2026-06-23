#[path = "date/pre_runtime_15.rs"]
mod pre_runtime_15;
#[path = "date/runtime_15.rs"]
mod runtime_15;

pub(super) fn expected_date_for_slice(slice: &str) -> &'static str {
    if let Some(date) = runtime_15::expected_date_for_slice(slice) {
        date
    } else {
        pre_runtime_15::expected_date_for_slice(slice)
    }
}
