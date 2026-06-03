use std::collections::BTreeMap;

use super::super::{
    assert_unique_dependency_row, bool_value, for_each_optional_feature,
    for_each_static_plugin_manifest, non_empty_string_array_values, non_empty_string_value,
};

#[test]
fn plugin_tomls_declare_optional_feature_dependencies() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");
        let package_capabilities =
            non_empty_string_array_values(table, relative_path, "top-level", "capabilities");

        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            let dependencies = feature
                .get("dependencies")
                .and_then(toml::Value::as_array)
                .unwrap_or_else(|| {
                    panic!(
                        "plugin manifest {relative_path:?} {feature_context} should declare dependency rows"
                    )
                });
            assert!(
                !dependencies.is_empty(),
                "plugin manifest {relative_path:?} {feature_context} should declare at least one dependency"
            );

            let mut primary_dependency_count = 0usize;
            for dependency in dependencies {
                let dependency = dependency.as_table().unwrap_or_else(|| {
                    panic!(
                        "plugin manifest {relative_path:?} {feature_context} dependency should be a table"
                    )
                });
                let dependency_plugin =
                    non_empty_string_value(dependency, relative_path, feature_context, "plugin_id");
                let dependency_capability = non_empty_string_value(
                    dependency,
                    relative_path,
                    feature_context,
                    "capability",
                );
                let primary = bool_value(dependency, relative_path, feature_context, "primary");

                if primary {
                    primary_dependency_count += 1;
                    assert_eq!(
                        dependency_plugin, package_id,
                        "plugin manifest {relative_path:?} {feature_context} primary dependency should point to owner plugin `{package_id}`"
                    );
                    assert!(
                        package_capabilities.contains(&dependency_capability),
                        "plugin manifest {relative_path:?} {feature_context} primary dependency capability `{dependency_capability}` should be a package capability"
                    );
                }
            }

            assert_eq!(
                primary_dependency_count, 1,
                "plugin manifest {relative_path:?} {feature_context} should declare exactly one primary dependency"
            );
        });
    });
}

#[test]
fn plugin_tomls_declare_unique_optional_feature_dependency_rows() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            let dependencies = feature
                .get("dependencies")
                .and_then(toml::Value::as_array)
                .unwrap_or_else(|| {
                    panic!(
                        "plugin manifest {relative_path:?} {feature_context} should declare dependency rows"
                    )
                });

            let mut dependency_rows = BTreeMap::new();
            for dependency in dependencies {
                let dependency = dependency.as_table().unwrap_or_else(|| {
                    panic!(
                        "plugin manifest {relative_path:?} {feature_context} dependency should be a table"
                    )
                });
                let dependency_plugin =
                    non_empty_string_value(dependency, relative_path, feature_context, "plugin_id");
                let dependency_capability = non_empty_string_value(
                    dependency,
                    relative_path,
                    feature_context,
                    "capability",
                );
                assert_unique_dependency_row(
                    &mut dependency_rows,
                    dependency_plugin,
                    dependency_capability,
                    format!(
                        "{feature_context} dependency `{dependency_plugin}` in {}",
                        relative_path.display()
                    ),
                );
            }
        });
    });
}
