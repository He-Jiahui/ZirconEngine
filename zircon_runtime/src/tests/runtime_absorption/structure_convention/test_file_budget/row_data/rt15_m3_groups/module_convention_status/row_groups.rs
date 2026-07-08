use super::*;

const MODULE_CONVENTION_STATUS_CHILD_ROWS: &[(&str, &str, &str, &str)] = &[
    (
        "status_rows",
        MODULE_CONVENTION_STATUS_STATUS_ROWS_PATH,
        "module_convention_status/status_rows.rs",
        "Runtime 15 M3 module-convention status row-data child-owner split",
    ),
    (
        "frontmatter_and_gate_rows",
        MODULE_CONVENTION_STATUS_FRONTMATTER_AND_GATE_ROWS_PATH,
        "module_convention_status/frontmatter_and_gate_rows.rs",
        "Runtime 15 M3 module convention module-doc frontmatter uniqueness guard",
    ),
    (
        "structure_guard_rows",
        MODULE_CONVENTION_STATUS_STRUCTURE_GUARD_ROWS_PATH,
        "module_convention_status/structure_guard_rows.rs",
        "Runtime 15 M3 generated-code guard folder-backed split",
    ),
    (
        "audit_rows",
        MODULE_CONVENTION_STATUS_AUDIT_ROWS_PATH,
        "module_convention_status/audit_rows.rs",
        "Runtime 15 M3 module convention gate output contract",
    ),
    (
        "row_data_owner",
        MODULE_CONVENTION_STATUS_ROW_DATA_OWNER_ROWS_PATH,
        "module_convention_status/row_data_owner.rs",
        "Runtime 15 M3 module-convention status row-data owner child split",
    ),
];

pub(super) fn assert_module_convention_status_parent_delegates_to_children() {
    let parent = read_runtime_src(MODULE_CONVENTION_STATUS_ROW_DATA_PATH);

    for (module_name, path, path_attr_suffix, representative_row) in
        MODULE_CONVENTION_STATUS_CHILD_ROWS
    {
        let path_attr = format!("#[path = \"{path_attr_suffix}\"]");
        let module_mount = format!("mod {module_name};");
        assert_contains_all(
            "module-convention status route mounts child row owner",
            &parent,
            &[path_attr.as_str(), module_mount.as_str()],
        );
        assert!(
            !parent.contains(representative_row),
            "module_convention_status.rs should route {representative_row} instead of owning it"
        );

        let child = read_runtime_src(path);
        assert_contains_all(
            path,
            &child,
            &[
                "pub(super) const EXPECTED_STATUS_OUTPUT_SLICES",
                *representative_row,
            ],
        );
    }
}
