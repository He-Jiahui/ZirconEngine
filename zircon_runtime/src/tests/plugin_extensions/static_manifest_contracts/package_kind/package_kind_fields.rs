use std::path::Path;

pub(super) fn package_kind_value<'a>(table: &'a toml::Table, relative_path: &Path) -> &'a str {
    let Some(package_kind) = table.get("package_kind") else {
        return "standard";
    };
    let package_kind = package_kind.as_str().unwrap_or_else(|| {
        panic!("plugin manifest {relative_path:?} package_kind should be a string")
    });
    assert_eq!(
        package_kind.trim(),
        package_kind,
        "plugin manifest {relative_path:?} package_kind `{package_kind}` should not have leading or trailing whitespace"
    );
    assert!(
        !package_kind.is_empty(),
        "plugin manifest {relative_path:?} package_kind should not be empty when declared"
    );
    package_kind
}

pub(super) fn table_array_row_count(
    table: &toml::Table,
    relative_path: &Path,
    field_name: &str,
) -> usize {
    let Some(rows) = table.get(field_name) else {
        return 0;
    };
    let rows = rows.as_array().unwrap_or_else(|| {
        panic!("plugin manifest {relative_path:?} {field_name} should be an array")
    });
    for (index, row) in rows.iter().enumerate() {
        assert!(
            row.as_table().is_some(),
            "plugin manifest {relative_path:?} {field_name} row {index} should be a table"
        );
    }
    rows.len()
}
