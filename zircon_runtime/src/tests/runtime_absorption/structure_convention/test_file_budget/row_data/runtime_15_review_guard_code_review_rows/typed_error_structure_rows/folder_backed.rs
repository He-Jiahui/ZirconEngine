use super::*;

pub(super) fn assert_typed_error_structure_rows_guard_is_folder_backed() {
    let route_source = read_runtime_src(TYPED_ERROR_STRUCTURE_ROWS_STATUS_OUTPUT_GUARD_PATH);

    for (module_name, path, marker) in TYPED_ERROR_STRUCTURE_ROWS_STATUS_OUTPUT_GUARD_CHILDREN {
        let module_mount = format!("#[path = \"typed_error_structure_rows/{module_name}.rs\"]");
        assert_contains_all(
            "typed-error structure row-data guard route mounts folder-backed children",
            &route_source,
            &[module_mount.as_str(), *marker],
        );
        let child_source = read_runtime_src(path);
        assert_contains_all(
            "typed-error structure row-data guard child keeps representative marker",
            &child_source,
            &[*marker],
        );
    }
    for forbidden in [
        "let row_data_owner = read_runtime_src",
        "let status_map = read_runtime_src(REVIEW_GUARD_STATUS_MAP_PATH)",
        "assert_status_doc_paths_rows_are_child_backed();",
        "Runtime 15 M3 typed-error status-doc paths child split",
    ] {
        assert!(
            !route_source.contains(forbidden),
            "typed_error_structure_rows.rs should delegate {forbidden} to child guard files"
        );
    }
}
