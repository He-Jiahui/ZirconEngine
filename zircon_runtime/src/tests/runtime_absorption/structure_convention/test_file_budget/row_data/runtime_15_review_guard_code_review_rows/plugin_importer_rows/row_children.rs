use super::*;

const PLUGIN_IMPORTER_ROW_DATA_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "review_guards",
        PLUGIN_IMPORTER_REVIEW_GUARDS_PATH,
        "PLUGIN_IMPORTER_DX_STRUCTURE_GUARD_CHILD_OWNER_SPLIT",
    ),
    (
        "row_data_owner",
        PLUGIN_IMPORTER_ROW_DATA_OWNER_PATH,
        "PLUGIN_IMPORTER_ROWS_ROW_DATA_OWNER_CHILD_SPLIT",
    ),
    (
        "source_inventory",
        PLUGIN_IMPORTER_SOURCE_INVENTORY_PATH,
        "PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_GUARD_CHILD_OWNER_SPLIT",
    ),
    (
        "status_docs",
        PLUGIN_IMPORTER_STATUS_DOCS_PATH,
        "PLUGIN_IMPORTER_DX_STATUS_DOCS_CHILD_OWNER_SPLIT",
    ),
    (
        "structure_assertions",
        PLUGIN_IMPORTER_STRUCTURE_ASSERTIONS_PATH,
        "PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_GUARD_CHILD_OWNER_SPLIT",
    ),
];

pub(super) fn assert_plugin_importer_row_data_children_are_current() {
    let parent = read_runtime_src(PLUGIN_IMPORTER_ROWS_PATH);
    let child_blob = PLUGIN_IMPORTER_ROW_DATA_CHILDREN
        .iter()
        .map(|(_, path, _)| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n");

    for (module_name, path, representative_anchor) in PLUGIN_IMPORTER_ROW_DATA_CHILDREN {
        let module_mount = format!("#[path = \"plugin_importer_rows/{module_name}.rs\"]");
        let module_reference = format!("{module_name}::");
        assert_contains_all(
            "plugin-importer row-data parent mounts every topic child",
            &parent,
            &[module_mount.as_str(), module_reference.as_str()],
        );
        let child_source = read_runtime_src(path);
        assert_contains_all(path, &child_source, &[*representative_anchor]);
    }
    assert!(
        !parent.contains("tests/runtime_absorption/code_review_findings/plugin_importer_dx.rs"),
        "plugin_importer_rows.rs should route topic children instead of owning row anchors directly"
    );
    assert_contains_all(
        "plugin-importer row-data parent registers the status row",
        &parent,
        &[
            PLUGIN_IMPORTER_ROWS_ROW_DATA_STATUS_NAME,
            "row_data_owner::PLUGIN_IMPORTER_ROWS_ROW_DATA_OWNER_CHILD_SPLIT",
        ],
    );
    assert_contains_all(
        "plugin-importer row-data children own representative rows",
        &child_blob,
        &[
            "runtime_15_code_review_findings_plugin_importer_dx_structure_guard_child_owner_split_static_passed_cargo_deferred",
            "runtime_15_plugin_importer_dx_status_docs_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_plugin_importer_dx_source_inventory_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_plugin_importer_dx_review_mounts_guard_folder_backed_static_passed_cargo_deferred",
            "runtime_15_plugin_importer_d13_sdk_review_guard_child_owner_split_static_passed_cargo_deferred",
            PLUGIN_IMPORTER_ROWS_ROW_DATA_STATUS_ID,
            PLUGIN_IMPORTER_ROWS_ROW_DATA_GUARD_NAME,
        ],
    );
}
