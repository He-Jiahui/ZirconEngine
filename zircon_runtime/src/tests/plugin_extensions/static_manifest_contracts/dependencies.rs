use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::{
    for_each_optional_feature, for_each_static_plugin_manifest, non_empty_string_array_values,
    non_empty_string_value,
};

struct StaticPackageCapabilities {
    capabilities: BTreeSet<String>,
}

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
            visit_optional_feature_dependency_rows(
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
            visit_optional_feature_dependency_rows(
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

#[test]
fn plugin_tomls_declare_dependency_provider_ids_are_tokens() {
    for_each_static_plugin_manifest(|relative_path, table| {
        visit_package_dependency_rows(table, relative_path, &mut |dependency_id, _| {
            assert_dependency_provider_id_token(
                relative_path,
                &format!("top-level dependency `{dependency_id}`"),
                "id",
                dependency_id,
            );
        });

        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            visit_optional_feature_dependency_rows(
                feature,
                relative_path,
                feature_context,
                &mut |dependency_plugin, _| {
                    assert_dependency_provider_id_token(
                        relative_path,
                        &format!("{feature_context} dependency `{dependency_plugin}`"),
                        "plugin_id",
                        dependency_plugin,
                    );
                },
            );
        });
    });
}

#[test]
fn plugin_tomls_declare_required_capability_gates_are_declared_or_host_owned() {
    let declared_capabilities = static_declared_capabilities();

    for_each_static_plugin_manifest(|relative_path, table| {
        visit_option_required_capabilities(table, relative_path, &mut |context, capability| {
            assert_declared_or_host_capability(
                relative_path,
                &context,
                capability,
                &declared_capabilities,
            );
        });
        visit_asset_importer_required_capabilities(
            table,
            relative_path,
            &mut |context, capability| {
                assert_declared_or_host_capability(
                    relative_path,
                    &context,
                    capability,
                    &declared_capabilities,
                );
            },
        );
    });
}

fn static_package_capabilities() -> BTreeMap<String, StaticPackageCapabilities> {
    let mut package_capabilities = BTreeMap::new();

    for_each_static_plugin_manifest(|relative_path, table| {
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");
        let mut capabilities = BTreeSet::new();

        for capability in
            non_empty_string_array_values(table, relative_path, "top-level", "capabilities")
        {
            capabilities.insert(capability.to_string());
        }
        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            for capability in non_empty_string_array_values(
                feature,
                relative_path,
                feature_context,
                "capabilities",
            ) {
                capabilities.insert(capability.to_string());
            }
        });

        package_capabilities.insert(
            package_id.to_string(),
            StaticPackageCapabilities { capabilities },
        );
    });

    package_capabilities
}

fn static_declared_capabilities() -> BTreeSet<String> {
    let mut capabilities = BTreeSet::new();

    for package in static_package_capabilities().into_values() {
        capabilities.extend(package.capabilities);
    }

    capabilities
}

fn visit_package_dependency_rows(
    table: &toml::Table,
    relative_path: &Path,
    visit: &mut impl FnMut(&str, &str),
) {
    let Some(dependencies) = table.get("dependencies") else {
        return;
    };
    let dependencies = dependencies.as_array().unwrap_or_else(|| {
        panic!("plugin manifest {relative_path:?} dependencies should be an array")
    });

    for dependency in dependencies {
        let dependency = dependency.as_table().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} dependency should be a table")
        });
        let dependency_id =
            non_empty_string_value(dependency, relative_path, "top-level dependency", "id");
        let capability = non_empty_string_value(
            dependency,
            relative_path,
            &format!("top-level dependency `{dependency_id}`"),
            "capability",
        );
        visit(dependency_id, capability);
    }
}

fn visit_optional_feature_dependency_rows(
    feature: &toml::Table,
    relative_path: &Path,
    feature_context: &str,
    visit: &mut impl FnMut(&str, &str),
) {
    let Some(dependencies) = feature.get("dependencies") else {
        return;
    };
    let dependencies = dependencies.as_array().unwrap_or_else(|| {
        panic!(
            "plugin manifest {relative_path:?} {feature_context} dependencies should be an array"
        )
    });

    for dependency in dependencies {
        let dependency = dependency.as_table().unwrap_or_else(|| {
            panic!(
                "plugin manifest {relative_path:?} {feature_context} dependency should be a table"
            )
        });
        let dependency_plugin =
            non_empty_string_value(dependency, relative_path, feature_context, "plugin_id");
        let capability =
            non_empty_string_value(dependency, relative_path, feature_context, "capability");
        visit(dependency_plugin, capability);
    }
}

fn visit_option_required_capabilities(
    table: &toml::Table,
    relative_path: &Path,
    visit: &mut impl FnMut(String, &str),
) {
    let Some(options) = table.get("options") else {
        return;
    };
    let options = options
        .as_array()
        .unwrap_or_else(|| panic!("plugin manifest {relative_path:?} options should be an array"));

    for option in options {
        let option = option.as_table().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} option should be a table")
        });
        let key = non_empty_string_value(option, relative_path, "plugin option", "key");
        if option.get("required_capability").is_some() {
            let capability = non_empty_string_value(
                option,
                relative_path,
                &format!("plugin option `{key}`"),
                "required_capability",
            );
            visit(
                format!("plugin option `{key}` required_capability"),
                capability,
            );
        }
    }
}

fn visit_asset_importer_required_capabilities(
    table: &toml::Table,
    relative_path: &Path,
    visit: &mut impl FnMut(String, &str),
) {
    let Some(importers) = table.get("asset_importers") else {
        return;
    };
    let importers = importers.as_array().unwrap_or_else(|| {
        panic!("plugin manifest {relative_path:?} asset_importers should be an array")
    });

    for importer in importers {
        let importer = importer.as_table().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} asset importer should be a table")
        });
        let importer_id = non_empty_string_value(importer, relative_path, "asset importer", "id");
        let importer_context = format!("asset importer `{importer_id}`");
        for capability in non_empty_string_array_values(
            importer,
            relative_path,
            &importer_context,
            "required_capabilities",
        ) {
            visit(
                format!("{importer_context} required_capabilities"),
                capability,
            );
        }
    }
}

fn assert_declared_dependency_capability(
    relative_path: &Path,
    context: &str,
    capability: &str,
    target: &StaticPackageCapabilities,
) {
    assert!(
        target.capabilities.contains(capability),
        "plugin manifest {relative_path:?} {context} capability `{capability}` should be declared by the referenced static plugin package or one of its optional features"
    );
}

fn assert_host_dependency_capability(relative_path: &Path, context: &str, capability: &str) {
    assert!(
        capability.starts_with("runtime.module.") || capability.starts_with("runtime.capability."),
        "plugin manifest {relative_path:?} {context} capability `{capability}` references no static plugin package and should use a runtime.module.* or runtime.capability.* host namespace"
    );
}

fn assert_dependency_provider_id_token(
    relative_path: &Path,
    context: &str,
    field_name: &str,
    provider_id: &str,
) {
    assert_eq!(
        provider_id.trim(),
        provider_id,
        "plugin manifest {relative_path:?} {context} `{field_name}` `{provider_id}` should not have leading or trailing whitespace"
    );
    assert!(
        provider_id
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase()),
        "plugin manifest {relative_path:?} {context} `{field_name}` `{provider_id}` should start with a lowercase ASCII letter"
    );
    assert!(
        provider_id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'),
        "plugin manifest {relative_path:?} {context} `{field_name}` `{provider_id}` should contain only lowercase ASCII letters, digits, or underscores"
    );
    assert!(
        !provider_id.ends_with('_') && !provider_id.contains("__"),
        "plugin manifest {relative_path:?} {context} `{field_name}` `{provider_id}` should not end with an underscore or contain repeated underscores"
    );
}

fn assert_declared_or_host_capability(
    relative_path: &Path,
    context: &str,
    capability: &str,
    declared_capabilities: &BTreeSet<String>,
) {
    assert!(
        declared_capabilities.contains(capability) || is_host_required_capability(capability),
        "plugin manifest {relative_path:?} {context} `{capability}` should reference a declared static package/feature capability or an explicitly host-owned capability"
    );
}

fn is_host_required_capability(capability: &str) -> bool {
    capability.starts_with("runtime.capability.") || capability == "runtime.asset.importer.native"
}
