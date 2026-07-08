pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 typed-error status-doc paths status-current child split" => Some(
            "runtime_15_typed_error_status_doc_paths_status_current_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc root paths folder-backed split" => Some(
            "runtime_15_typed_error_status_doc_root_paths_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc root paths folder-backed guard folder-backed split" => Some(
            "runtime_15_typed_error_status_doc_root_paths_folder_backed_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc status-slices folder-backed split" => Some(
            "runtime_15_typed_error_status_doc_status_slices_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc status-slices folder-backed guard folder-backed split" => Some(
            "runtime_15_typed_error_status_doc_status_slices_folder_backed_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc paths status-current sources child split" => Some(
            "runtime_15_typed_error_status_doc_paths_status_current_sources_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc paths status-current sources guard folder-backed split" => Some(
            "runtime_15_typed_error_status_doc_paths_status_current_sources_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc paths child inventory child split" => Some(
            "runtime_15_typed_error_status_doc_paths_child_inventory_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc paths child inventory split-layout folder-backed split" => Some(
            "runtime_15_typed_error_status_doc_paths_child_inventory_split_layout_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc paths child inventory split-layout guard folder-backed split" => Some(
            "runtime_15_typed_error_status_doc_paths_child_inventory_split_layout_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc paths child inventory split-layout sources child split" => Some(
            "runtime_15_typed_error_status_doc_paths_child_inventory_split_layout_sources_child_split_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc paths child inventory split-layout sources guard folder-backed split" => Some(
            "runtime_15_typed_error_status_doc_paths_child_inventory_split_layout_sources_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc paths child inventory split-layout status mirrors guard folder-backed split" => Some(
            "runtime_15_typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc paths child inventory split-layout status mirrors status-current folder-backed split" => Some(
            "runtime_15_typed_error_status_doc_paths_child_inventory_split_layout_status_mirrors_status_current_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc paths status-current split-layout folder-backed split" => Some(
            "runtime_15_typed_error_status_doc_paths_status_current_split_layout_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc paths status-current split-layout guard folder-backed split" => Some(
            "runtime_15_typed_error_status_doc_paths_status_current_split_layout_guard_folder_backed_static_passed_cargo_deferred",
        ),
        "Runtime 15 M3 typed-error status-doc paths status-current split-layout status mirrors guard folder-backed split" => Some(
            "runtime_15_typed_error_status_doc_paths_status_current_split_layout_status_mirrors_guard_folder_backed_static_passed_cargo_deferred",
        ),
        _ => None,
    }
}
