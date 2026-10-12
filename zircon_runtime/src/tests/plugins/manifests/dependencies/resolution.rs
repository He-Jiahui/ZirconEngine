use super::super::{
    for_each_feature_extension, for_each_optional_feature, for_each_static_plugin_manifest,
    visit_feature_dependency_rows, visit_package_dependency_rows,
};
use super::capabilities::static_package_capabilities;
use super::capability_assertions::{
    assert_declared_dependency_capability, assert_host_dependency_capability,
};

#[test]
fn plugin_tomls_declare_dependency_capabilities_reference_static_packages() {
    let package_capabilities = static_package_capabilities();

    for_each_static_plugin_manifest(|relative_path, table| {
        visit_package_dependency_rows(table, relative_path, &mut |dependency_id, capability| {
            if let Some(target) = package_capabilities.get(dependency_id) {
                assert_declared_dependency_capability(
                    relative_path,
                    &format!("top-level dependency `{dependency_id}`"),
                    capability,
                    target,
                );
            }
        });

        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            visit_feature_dependency_rows(
                feature,
                relative_path,
                feature_context,
                &mut |dependency_plugin, capability| {
                    if let Some(target) = package_capabilities.get(dependency_plugin) {
                        assert_declared_dependency_capability(
                            relative_path,
                            &format!("{feature_context} dependency `{dependency_plugin}`"),
                            capability,
                            target,
                        );
                    }
                },
            );
        });
        for_each_feature_extension(table, relative_path, &mut |feature, feature_context| {
            visit_feature_dependency_rows(
                feature,
                relative_path,
                feature_context,
                &mut |dependency_plugin, capability| {
                    if let Some(target) = package_capabilities.get(dependency_plugin) {
                        assert_declared_dependency_capability(
                            relative_path,
                            &format!("{feature_context} dependency `{dependency_plugin}`"),
                            capability,
                            target,
                        );
                    }
                },
            );
        });
    });
}

#[test]
fn plugin_tomls_declare_external_dependency_capabilities_use_host_namespaces() {
    let package_capabilities = static_package_capabilities();

    for_each_static_plugin_manifest(|relative_path, table| {
        visit_package_dependency_rows(table, relative_path, &mut |dependency_id, capability| {
            if !package_capabilities.contains_key(dependency_id) {
                assert_host_dependency_capability(
                    relative_path,
                    &format!("top-level dependency `{dependency_id}`"),
                    capability,
                );
            }
        });

        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            visit_feature_dependency_rows(
                feature,
                relative_path,
                feature_context,
                &mut |dependency_plugin, capability| {
                    if !package_capabilities.contains_key(dependency_plugin) {
                        assert_host_dependency_capability(
                            relative_path,
                            &format!("{feature_context} dependency `{dependency_plugin}`"),
                            capability,
                        );
                    }
                },
            );
        });
        for_each_feature_extension(table, relative_path, &mut |feature, feature_context| {
            visit_feature_dependency_rows(
                feature,
                relative_path,
                feature_context,
                &mut |dependency_plugin, capability| {
                    if !package_capabilities.contains_key(dependency_plugin) {
                        assert_host_dependency_capability(
                            relative_path,
                            &format!("{feature_context} dependency `{dependency_plugin}`"),
                            capability,
                        );
                    }
                },
            );
        });
    });
}
