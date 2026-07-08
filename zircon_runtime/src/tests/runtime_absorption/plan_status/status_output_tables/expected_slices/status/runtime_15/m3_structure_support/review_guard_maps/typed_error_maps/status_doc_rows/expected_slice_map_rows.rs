pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 typed-error status-doc expected-slice rows folder-backed split" => Some(
            "runtime_15_typed_error_status_doc_expected_slice_rows_folder_backed_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
