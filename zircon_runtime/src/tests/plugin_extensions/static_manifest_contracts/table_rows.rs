use std::path::Path;

pub(super) fn required_table_array<'a>(
    table: &'a toml::Table,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) -> Vec<&'a toml::Table> {
    let rows =
        optional_table_array(table, relative_path, context, field_name).unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} {context} should declare `{field_name}` rows")
        });
    assert!(
        !rows.is_empty(),
        "plugin manifest {relative_path:?} {context} `{field_name}` rows should not be empty"
    );
    rows
}

pub(super) fn optional_table_array<'a>(
    table: &'a toml::Table,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) -> Option<Vec<&'a toml::Table>> {
    let value = table.get(field_name)?;
    Some(
        value
            .as_array()
            .unwrap_or_else(|| {
                panic!(
                    "plugin manifest {relative_path:?} {context} `{field_name}` should be an array"
                )
            })
            .iter()
            .map(|row| {
                row.as_table().unwrap_or_else(|| {
                    panic!(
                        "plugin manifest {relative_path:?} {context} `{field_name}` row should be a table"
                    )
                })
            })
            .collect(),
    )
}
