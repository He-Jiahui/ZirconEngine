pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 Runtime 07/script expected-slice maps folder-backed split" => Some(
            "runtime_15_runtime_07_script_expected_slice_maps_folder_backed_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
