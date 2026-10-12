use std::path::Path;

use super::super::non_empty_string_value;

pub(super) fn capability_status_array<'a>(
    table: &'a toml::Table,
    relative_path: &Path,
) -> Option<&'a Vec<toml::Value>> {
    let Some(statuses) = table.get("capability_statuses") else {
        return None;
    };
    Some(statuses.as_array().unwrap_or_else(|| {
        panic!("plugin manifest {relative_path:?} capability_statuses should be an array")
    }))
}

pub(super) fn visit_capability_status_rows(
    table: &toml::Table,
    relative_path: &Path,
    visit: &mut impl FnMut(&str, String),
) {
    let Some(statuses) = capability_status_array(table, relative_path) else {
        return;
    };

    for status in statuses {
        let status = status.as_table().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} capability status should be a table")
        });
        let capability =
            non_empty_string_value(status, relative_path, "capability status", "capability");
        visit(capability, format!("capability status `{capability}`"));
    }
}
