use super::*;

pub(super) fn assert_typed_error_structure_assertions_guard_is_folder_backed() {
    let route_source = read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_GUARD_PATH);

    for (module_name, path, marker) in TYPED_ERROR_STRUCTURE_ASSERTIONS_GUARD_CHILDREN {
        let module_mount = format!("#[path = \"typed_error_structure/{module_name}.rs\"]");
        assert_contains_all(
            "typed-error structure-assertions guard route mounts folder-backed children",
            &route_source,
            &[module_mount.as_str(), &format!("mod {module_name};")],
        );
        assert_contains_all(
            "typed-error structure-assertions guard child keeps representative marker",
            &read_runtime_src(path),
            &[*marker],
        );
    }
    for forbidden in [
        "let review_guard_status_rows = read_runtime_src",
        "let status_map = read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_STATUS_MAP_PATH)",
        "for moved_anchor in [",
        "typed-error structure row aggregate still consumes structure-assertions exports",
    ] {
        assert!(
            !route_source.contains(forbidden),
            "typed_error_structure_assertions.rs should delegate {forbidden} to child guard files"
        );
    }
}
