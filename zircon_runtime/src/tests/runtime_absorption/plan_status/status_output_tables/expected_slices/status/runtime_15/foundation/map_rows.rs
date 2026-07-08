const SLICE: &str = "Runtime 15 M3 foundation expected-slice maps folder-backed split";
const STATUS: &str =
    "runtime_15_foundation_expected_slice_maps_folder_backed_static_passed_cargo_deferred";

pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if slice == SLICE {
        Some(STATUS)
    } else {
        None
    }
}
