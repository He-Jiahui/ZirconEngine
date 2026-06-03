use std::path::Path;

use super::super::non_empty_string_value;

const COORDINATE_FIELDS: [&str; 3] = ["package_prefix", "package_company", "package_name"];

pub(super) fn declares_any_coordinate_field(table: &toml::Table) -> bool {
    COORDINATE_FIELDS
        .iter()
        .any(|field_name| table.get(*field_name).is_some())
}

pub(super) fn resolved_package_id(table: &toml::Table, relative_path: &Path) -> String {
    if declares_any_coordinate_field(table) {
        let package_prefix = non_empty_string_value(
            table,
            relative_path,
            "package coordinates",
            "package_prefix",
        );
        let package_company = non_empty_string_value(
            table,
            relative_path,
            "package coordinates",
            "package_company",
        );
        let package_name =
            non_empty_string_value(table, relative_path, "package coordinates", "package_name");
        return format!("{package_prefix}.{package_company}.{package_name}");
    }

    non_empty_string_value(table, relative_path, "top-level", "id").to_string()
}
