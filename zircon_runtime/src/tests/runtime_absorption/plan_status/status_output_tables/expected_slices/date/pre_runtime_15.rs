#[path = "pre_runtime_15/runtime_01_05.rs"]
mod runtime_01_05;
#[path = "pre_runtime_15/runtime_06_10.rs"]
mod runtime_06_10;
#[path = "pre_runtime_15/runtime_11_14.rs"]
mod runtime_11_14;

pub(super) fn expected_date_for_slice(slice: &str) -> &'static str {
    if let Some(value) = runtime_01_05::expected_date_for_slice(slice) {
        value
    } else if let Some(value) = runtime_06_10::expected_date_for_slice(slice) {
        value
    } else if let Some(value) = runtime_11_14::expected_date_for_slice(slice) {
        value
    } else {
        "2026-06-14"
    }
}
