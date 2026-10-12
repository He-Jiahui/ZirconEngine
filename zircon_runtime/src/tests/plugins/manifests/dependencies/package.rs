use std::collections::BTreeMap;

use super::super::{
    assert_unique_dependency_row, bool_value, for_each_static_plugin_manifest,
    non_empty_string_array_values, non_empty_string_value,
};

#[test]
fn plugin_tomls_declare_package_dependencies() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let Some(dependencies) = table.get("dependencies").and_then(toml::Value::as_array) else {
            return;
        };

        assert!(
            !dependencies.is_empty(),
            "plugin manifest {relative_path:?} dependencies should not be empty when declared"
        );

        for dependency in dependencies {
            let dependency = dependency.as_table().unwrap_or_else(|| {
                panic!("plugin manifest {relative_path:?} dependency should be a table")
            });
            let dependency_id =
                non_empty_string_value(dependency, relative_path, "top-level dependency", "id");
            bool_value(
                dependency,
                relative_path,
                &format!("top-level dependency `{dependency_id}`"),
                "required",
            );
            if !dependency.contains_key("capability") {
                let interfaces = non_empty_string_array_values(
                    dependency,
                    relative_path,
                    &format!("top-level dependency `{dependency_id}`"),
                    "interfaces",
                );
                assert!(
                    !interfaces.is_empty(),
                    "plugin manifest {relative_path:?} top-level dependency `{dependency_id}` interfaces should not be empty when capability is absent"
                );
            } else {
                non_empty_string_value(
                    dependency,
                    relative_path,
                    &format!("top-level dependency `{dependency_id}`"),
                    "capability",
                );
            }
        }
    });
}

#[test]
fn plugin_tomls_declare_unique_package_dependency_rows() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let Some(dependencies) = table.get("dependencies").and_then(toml::Value::as_array) else {
            return;
        };

        let mut dependency_rows = BTreeMap::new();
        for dependency in dependencies {
            let dependency = dependency.as_table().unwrap_or_else(|| {
                panic!("plugin manifest {relative_path:?} dependency should be a table")
            });
            let dependency_id =
                non_empty_string_value(dependency, relative_path, "top-level dependency", "id");
            let dependency_capability = dependency
                .get("capability")
                .and_then(toml::Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| {
                    non_empty_string_array_values(
                        dependency,
                        relative_path,
                        &format!("top-level dependency `{dependency_id}`"),
                        "interfaces",
                    )
                    .join(",")
                });
            assert_unique_dependency_row(
                &mut dependency_rows,
                dependency_id,
                dependency_capability.as_str(),
                format!(
                    "top-level dependency `{dependency_id}` in {}",
                    relative_path.display()
                ),
            );
        }
    });
}
