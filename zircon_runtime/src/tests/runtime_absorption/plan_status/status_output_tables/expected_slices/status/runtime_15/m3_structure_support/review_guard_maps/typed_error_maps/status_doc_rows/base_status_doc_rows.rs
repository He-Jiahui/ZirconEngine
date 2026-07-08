pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 typed-error structure status-doc guard child-owner split" => Some(
            "runtime_15_typed_error_structure_status_docs_child_owner_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error structure status-doc guard folder-backed split" => Some(
            "runtime_15_typed_error_structure_status_docs_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc doc mirrors folder-backed split" => Some(
            "runtime_15_typed_error_status_doc_mirrors_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc doc mirrors source helper child split" => Some(
            "runtime_15_typed_error_status_doc_mirrors_source_helper_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc source helper child split" => Some(
            "runtime_15_typed_error_status_doc_source_helper_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc paths child split" => Some(
            "runtime_15_typed_error_status_doc_paths_child_split_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
