use super::*;

pub(super) fn assert_typed_error_status_docs_guard_is_folder_backed() {
    let route_source = read_runtime_src(TYPED_ERROR_STATUS_DOCS_GUARD_PATH);

    for (module_name, path, marker) in TYPED_ERROR_STATUS_DOCS_GUARD_CHILDREN {
        let module_mount = format!("#[path = \"typed_error_status/{module_name}.rs\"]");
        assert_contains_all(
            "typed-error status-doc guard route mounts folder-backed children",
            &route_source,
            &[module_mount.as_str(), &format!("mod {module_name};")],
        );
        assert_contains_all(
            "typed-error status-doc guard child owns representative marker",
            &read_runtime_src(path),
            &[*marker],
        );
    }
    for forbidden in [
        "let review_guard_status_rows = read_runtime_src",
        "let status_map = read_runtime_src(TYPED_ERROR_STATUS_DOCS_ROW_DATA_STATUS_MAP_PATH)",
        "for moved_anchor in [",
        "typed-error structure row aggregate still consumes status-doc row exports",
    ] {
        assert!(
            !route_source.contains(forbidden),
            "typed_error_status_docs.rs should delegate {forbidden} to child guard files"
        );
    }
}
