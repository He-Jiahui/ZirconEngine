#[path = "status/pre_runtime_15.rs"]
mod pre_runtime_15;
#[path = "status/runtime_15.rs"]
mod runtime_15;

pub(super) fn expected_status_for_slice(slice: &str) -> &'static str {
    if let Some(status) = runtime_15::expected_status_for_slice(slice) {
        status
    } else {
        pre_runtime_15::expected_status_for_slice(slice)
    }
}
