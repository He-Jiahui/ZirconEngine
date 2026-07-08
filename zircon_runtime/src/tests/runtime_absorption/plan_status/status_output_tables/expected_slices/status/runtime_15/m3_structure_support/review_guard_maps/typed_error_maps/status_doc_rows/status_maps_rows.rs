pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 typed-error status-doc status maps child split" => Some(
            "runtime_15_typed_error_status_doc_status_maps_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc status maps status-current child split" => Some(
            "runtime_15_typed_error_status_doc_status_maps_status_current_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc status maps status-current sources child split" => Some(
            "runtime_15_typed_error_status_doc_status_maps_status_current_sources_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc status maps status-current sources guard folder-backed split" => Some(
            "runtime_15_typed_error_status_doc_status_maps_status_current_sources_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc status maps status-current split-layout guard folder-backed split" => Some(
            "runtime_15_typed_error_status_doc_status_maps_status_current_split_layout_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc status maps status-current split-layout sources child split" => Some(
            "runtime_15_typed_error_status_doc_status_maps_status_current_split_layout_sources_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc status maps status-current split-layout sources guard folder-backed split" => Some(
            "runtime_15_typed_error_status_doc_status_maps_status_current_split_layout_sources_guard_folder_backed_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
