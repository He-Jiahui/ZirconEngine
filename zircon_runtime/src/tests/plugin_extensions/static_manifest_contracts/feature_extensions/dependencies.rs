use std::collections::BTreeMap;

use super::{
    assert_lowercase_dot_namespace, assert_package_token, assert_unique_dependency_row, bool_value,
    for_each_static_plugin_manifest, non_empty_string_value, required_table_array,
    static_package_capabilities, visit_feature_extension_rows,
};

#[test]
fn plugin_tomls_declare_feature_extension_dependencies() {
    let package_capabilities = static_package_capabilities();

    for_each_static_plugin_manifest(|relative_path, table| {
        visit_feature_extension_rows(table, relative_path, &mut |feature, feature_context| {
            let owner_plugin_id =
                non_empty_string_value(feature, relative_path, feature_context, "owner_plugin_id");
            let dependencies =
                required_table_array(feature, relative_path, feature_context, "dependencies");

            let mut dependency_rows = BTreeMap::new();
            let mut primary_dependency_count = 0usize;
            for dependency in dependencies {
                let dependency_plugin =
                    non_empty_string_value(dependency, relative_path, feature_context, "plugin_id");
                assert_package_token(
                    relative_path,
                    feature_context,
                    "plugin_id",
                    dependency_plugin,
                );
                let dependency_capability = non_empty_string_value(
                    dependency,
                    relative_path,
                    feature_context,
                    "capability",
                );
                assert_lowercase_dot_namespace(
                    relative_path,
                    feature_context,
                    "capability",
                    dependency_capability,
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

                if let Some(target) = package_capabilities.get(dependency_plugin) {
                    assert!(
                        target.contains_declared(dependency_capability),
                        "plugin manifest {relative_path:?} {feature_context} dependency `{dependency_plugin}` capability `{dependency_capability}` should be declared by the referenced static plugin package or one of its feature rows"
                    );
                }

                if bool_value(dependency, relative_path, feature_context, "primary") {
                    primary_dependency_count += 1;
                    assert_eq!(
                        dependency_plugin, owner_plugin_id,
                        "plugin manifest {relative_path:?} {feature_context} primary dependency should point to owner plugin `{owner_plugin_id}`"
                    );
                    if let Some(owner) = package_capabilities.get(owner_plugin_id) {
                        assert!(
                            owner.package_capabilities.contains(dependency_capability),
                            "plugin manifest {relative_path:?} {feature_context} primary dependency capability `{dependency_capability}` should be a package capability declared by owner plugin `{owner_plugin_id}`"
                        );
                    }
                }
            }

            assert_eq!(
                primary_dependency_count, 1,
                "plugin manifest {relative_path:?} {feature_context} should declare exactly one primary dependency"
            );
        });
    });
}
