use super::*;

pub(super) fn assert_typed_error_status_doc_row_routes_are_child_backed() {
    let parent = read_runtime_src(TYPED_ERROR_STATUS_DOCS_ROW_DATA_PATH);
    let typed_error_rows = read_runtime_src(TYPED_ERROR_STRUCTURE_ROWS_PATH);

    for (module_name, marker) in TYPED_ERROR_STATUS_DOCS_ROW_DATA_CHILDREN {
        let child_path = typed_error_status_docs_row_data_child_path(module_name);
        let module_mount = format!("#[path = \"status/{module_name}.rs\"]");
        let re_export = format!("pub(super) use {module_name}::*;");
        assert_contains_all(
            "typed-error status-doc row-data route mounts child row groups",
            &parent,
            &[
                module_mount.as_str(),
                &format!("mod {module_name};"),
                re_export.as_str(),
            ],
        );
        assert_contains_all(
            "typed-error status-doc row-data child tree owns representative row group",
            &typed_error_status_docs_row_data_child_tree_blob(module_name),
            &[*marker, "Cargo gate deferred"],
        );
        assert!(
            !read_runtime_src(&child_path).contains("Runtime 15 M3 typed-error status-doc paths child inventory split-layout sources child split"),
            "{child_path} should route nested row groups instead of owning deep status anchors",
        );
    }

    for moved_anchor in [
        "pub(super) const STATUS_DOC_GUARD_FOLDER_BACKED_SPLIT",
        "pub(super) const STATUS_DOC_PATHS_CHILD_SPLIT",
        "pub(super) const STATUS_DOC_DELEGATION_CHILD_SPLIT",
        "pub(super) const STATUS_DOC_STATUS_MAPS_CHILD_SPLIT",
        "pub(super) const STATUS_DOC_STATUS_MIRRORS_CHILD_SPLIT",
        "runtime_15_typed_error_status_doc_status_maps_status_current_sources_child_split_static_passed_cargo_deferred",
        "Cargo gate deferred",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error status-doc row-data route should not own moved row anchor {moved_anchor}"
        );
    }

    assert_contains_all(
        "typed-error status-doc row-data child tree keeps row anchors",
        &typed_error_status_docs_row_data_child_blob(),
        &[
            "runtime_15_typed_error_structure_status_docs_folder_backed_static_passed_cargo_deferred",
            "runtime_15_typed_error_status_doc_paths_child_inventory_split_layout_sources_child_split_static_passed_cargo_deferred",
            "runtime_15_typed_error_status_doc_delegation_status_current_sources_child_split_static_passed_cargo_deferred",
            "runtime_15_typed_error_status_doc_status_maps_status_current_sources_child_split_static_passed_cargo_deferred",
            "runtime_15_typed_error_status_doc_status_mirrors_status_current_sources_child_split_static_passed_cargo_deferred",
            "runtime_15_typed_error_status_doc_status_mirrors_status_current_split_layout_sources_child_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "typed-error structure row aggregate still routes status-doc row-data owner",
        &typed_error_rows,
        &[
            "#[path = \"typed_error_structure_rows/status_docs.rs\"]",
            "mod status_docs;",
            "status_doc_path_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "status_doc_delegation_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "status_doc_status_maps_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "status_doc_status_mirrors_rows::EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
}
