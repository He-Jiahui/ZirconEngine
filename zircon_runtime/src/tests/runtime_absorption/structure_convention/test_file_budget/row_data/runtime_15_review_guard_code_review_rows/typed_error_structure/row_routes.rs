use super::*;

pub(super) fn assert_typed_error_structure_assertion_row_routes_are_child_backed() {
    let parent = read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTIONS_ROW_DATA_PATH);
    let typed_error_rows = read_runtime_src(TYPED_ERROR_STRUCTURE_ROWS_PATH);
    let core_rows = read_runtime_src(TYPED_ERROR_STRUCTURE_CORE_ROWS_PATH);
    let structure_assertion_rows = read_runtime_src(TYPED_ERROR_STRUCTURE_ASSERTION_ROWS_PATH);
    let child_blob = typed_error_structure_assertions_child_blob();

    for (module_name, marker) in TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILDREN {
        let child_path = typed_error_structure_assertions_child_path(module_name);
        let module_mount = format!("#[path = \"structure/{module_name}.rs\"]");
        let re_export = format!("pub(super) use {module_name}::*;");
        assert_contains_all(
            "typed-error structure-assertions row-data route mounts child row groups",
            &parent,
            &[
                module_mount.as_str(),
                &format!("mod {module_name};"),
                re_export.as_str(),
            ],
        );
        assert_contains_all(
            "typed-error structure-assertions child owns representative row group",
            &read_runtime_src(&child_path),
            &[*marker, "Cargo gate deferred"],
        );
    }

    for moved_anchor in [
        "pub(in super::super) const STRUCTURE_ASSERTIONS_GUARD_FOLDER_BACKED_SPLIT",
        "pub(in super::super) const CONVERGENCE_MOUNTS_GUARD_FOLDER_BACKED_SPLIT",
        "pub(in super::super) const NATIVE_PLUGIN_LOADER_GUARD_CHILD_OWNER_SPLIT",
        "pub(in super::super) const MOVED_GUARD_ABSENCE_CHILD_OWNER_SPLIT",
        "runtime_15_typed_error_native_plugin_loader_routes_child_split_static_passed_cargo_deferred",
        "Cargo gate deferred",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed-error structure-assertions row-data route should not own moved row anchor {moved_anchor}"
        );
    }

    assert_contains_all(
        "typed-error structure-assertions child tree keeps row anchors",
        &child_blob,
        &[
            "runtime_15_typed_error_structure_assertions_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_typed_error_convergence_mounts_root_inventory_child_split_static_passed_cargo_deferred",
            "runtime_15_typed_error_native_plugin_loader_routes_child_split_static_passed_cargo_deferred",
            "runtime_15_typed_error_moved_guard_absence_parent_backflow_child_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "typed-error structure row aggregate still routes structure-assertions row-data owners",
        &typed_error_rows,
        &[
            "#[path = \"typed_error_structure_rows/structure_assertions.rs\"]",
            "mod structure_assertions;",
            "#[path = \"typed_error_structure_rows/structure_assertion_rows.rs\"]",
            "mod structure_assertion_rows;",
            "structure_assertion_rows::EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "typed-error structure core rows still consume foundation/convergence exports",
        &core_rows,
        &[
            "super::structure_assertions::STRUCTURE_ASSERTIONS_GUARD_FOLDER_BACKED_SPLIT",
            "super::structure_assertions::CONVERGENCE_MOUNTS_GUARD_FOLDER_BACKED_SPLIT",
        ],
    );
    assert_contains_all(
        "typed-error structure-assertion rows still consume native/moved exports",
        &structure_assertion_rows,
        &[
            "super::structure_assertions::NATIVE_PLUGIN_LOADER_GUARD_CHILD_OWNER_SPLIT",
            "super::structure_assertions::MOVED_GUARD_ABSENCE_CHILD_OWNER_SPLIT",
        ],
    );
}
