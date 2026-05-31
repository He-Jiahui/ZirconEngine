use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

use super::{
    for_each_module_row, for_each_optional_feature, for_each_static_plugin_manifest,
    non_empty_string_array_values, non_empty_string_value, plugins_workspace_root,
    visit_module_rows,
};

#[test]
fn plugin_tomls_declare_capability_namespaces() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for capability in
            non_empty_string_array_values(table, relative_path, "top-level", "capabilities")
        {
            assert_capability_namespace(relative_path, "top-level capabilities", capability);
        }

        visit_capability_status_rows(table, relative_path, &mut |capability, context| {
            assert_capability_namespace(relative_path, &context, capability);
        });
        visit_package_dependency_rows(table, relative_path, &mut |capability, context| {
            assert_capability_namespace(relative_path, &context, capability);
        });
        visit_asset_importer_required_capabilities(
            table,
            relative_path,
            &mut |capability, context| {
                assert_capability_namespace(relative_path, &context, capability);
            },
        );
        visit_option_required_capabilities(table, relative_path, &mut |capability, context| {
            assert_capability_namespace(relative_path, &context, capability);
        });

        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            for capability in non_empty_string_array_values(
                feature,
                relative_path,
                feature_context,
                "capabilities",
            ) {
                assert_capability_namespace(relative_path, feature_context, capability);
            }
            visit_optional_feature_dependency_rows(
                feature,
                relative_path,
                feature_context,
                &mut |capability, context| {
                    assert_capability_namespace(relative_path, &context, capability);
                },
            );
        });

        for_each_module_row(table, relative_path, &mut |module, module_context| {
            let module_name = module
                .get("name")
                .and_then(toml::Value::as_str)
                .unwrap_or("<unknown>");
            let module_context = format!("{module_context} module `{module_name}`");
            for capability in non_empty_string_array_values(
                module,
                relative_path,
                &module_context,
                "capabilities",
            ) {
                assert_capability_namespace(relative_path, &module_context, capability);
            }
        });
    });
}

#[test]
fn plugin_tomls_declare_unique_capability_owners() {
    let mut capability_owners = BTreeMap::new();

    for_each_static_plugin_manifest(|relative_path, table| {
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");
        for capability in
            non_empty_string_array_values(table, relative_path, "top-level", "capabilities")
        {
            assert_unique_capability_owner(
                &mut capability_owners,
                capability,
                format!(
                    "top-level package `{package_id}` in {}",
                    relative_path.display()
                ),
            );
        }

        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            let feature_id = non_empty_string_value(feature, relative_path, feature_context, "id");
            for capability in non_empty_string_array_values(
                feature,
                relative_path,
                feature_context,
                "capabilities",
            ) {
                assert_unique_capability_owner(
                    &mut capability_owners,
                    capability,
                    format!(
                        "optional feature `{feature_id}` in {}",
                        relative_path.display()
                    ),
                );
            }
        });
    });
}

#[test]
fn plugin_tomls_declare_capability_statuses_reference_owned_capabilities() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let owned_capabilities = owned_package_and_feature_capabilities(table, relative_path);

        visit_capability_status_rows(table, relative_path, &mut |capability, context| {
            assert!(
                owned_capabilities.contains(capability),
                "plugin manifest {relative_path:?} {context} should reference a package or optional-feature capability declared by the same manifest"
            );
        });
    });
}

#[test]
fn plugin_tomls_declare_module_capabilities_stay_under_owner_namespace() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");

        visit_module_rows(
            table.get("modules"),
            relative_path,
            "package",
            &mut |module, module_context| {
                assert_module_capabilities_under_owner_namespace(
                    module,
                    relative_path,
                    module_context,
                    package_id,
                );
            },
        );

        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            let feature_id = non_empty_string_value(feature, relative_path, feature_context, "id");
            visit_module_rows(
                feature.get("modules"),
                relative_path,
                feature_context,
                &mut |module, module_context| {
                    assert_module_capabilities_under_owner_namespace(
                        module,
                        relative_path,
                        module_context,
                        feature_id,
                    );
                },
            );
        });
    });
}

fn assert_module_capabilities_under_owner_namespace(
    module: &toml::Table,
    relative_path: &Path,
    module_context: &str,
    owner_namespace: &str,
) {
    let module_name = non_empty_string_value(module, relative_path, module_context, "name");
    let module_context = format!("{module_context} module `{module_name}`");
    for capability in
        non_empty_string_array_values(module, relative_path, &module_context, "capabilities")
    {
        assert_capability_mentions_owner_namespace(
            relative_path,
            &module_context,
            capability,
            owner_namespace,
        );
    }
}

fn assert_capability_mentions_owner_namespace(
    relative_path: &Path,
    context: &str,
    capability: &str,
    owner_namespace: &str,
) {
    let capability_segments: Vec<_> = capability.split('.').collect();
    let owner_segments: Vec<_> = owner_namespace.split('.').collect();
    let contains_owner_segments = capability_segments
        .windows(owner_segments.len())
        .any(|segments| segments == owner_segments.as_slice());
    let contains_owner_prefixed_segment = owner_segments.len() == 1
        && capability_segments.iter().any(|segment| {
            matches!(
                segment.strip_prefix(owner_segments[0]),
                Some(suffix) if suffix.starts_with('_')
            )
        });

    assert!(
        contains_owner_segments || contains_owner_prefixed_segment,
        "plugin manifest {relative_path:?} {context} capability `{capability}` should stay under owner namespace `{owner_namespace}`"
                );
}

#[test]
fn plugin_tomls_declare_bevy_references_resolve_under_dev_bevy() {
    let plugins_root = plugins_workspace_root();
    let repo_root = plugins_root
        .parent()
        .expect("zircon_plugins workspace should be under the repository root");
    let bevy_root = Path::new("dev").join("bevy");

    for_each_static_plugin_manifest(|relative_path, table| {
        let Some(statuses) = table
            .get("capability_statuses")
            .and_then(toml::Value::as_array)
        else {
            return;
        };

        for status in statuses {
            let status = status.as_table().unwrap_or_else(|| {
                panic!("plugin manifest {relative_path:?} capability status should be a table")
            });
            let capability =
                non_empty_string_value(status, relative_path, "capability status", "capability");
            let context = format!("capability status `{capability}`");
            let Some(_) = status.get("bevy_references") else {
                continue;
            };

            for reference in
                non_empty_string_array_values(status, relative_path, &context, "bevy_references")
            {
                assert_bevy_reference_path(relative_path, &context, reference, &bevy_root);
                let reference_path = repo_root.join(reference);
                assert!(
                    reference_path.is_file(),
                    "plugin manifest {relative_path:?} {context} bevy reference `{reference}` should resolve to an existing file"
                );
            }
        }
    });
}

fn owned_package_and_feature_capabilities(
    table: &toml::Table,
    relative_path: &Path,
) -> BTreeSet<String> {
    let mut capabilities = BTreeSet::new();
    for capability in
        non_empty_string_array_values(table, relative_path, "top-level", "capabilities")
    {
        capabilities.insert(capability.to_string());
    }

    for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
        for capability in
            non_empty_string_array_values(feature, relative_path, feature_context, "capabilities")
        {
            capabilities.insert(capability.to_string());
        }
    });

    capabilities
}

fn assert_bevy_reference_path(
    relative_path: &Path,
    context: &str,
    reference: &str,
    bevy_root: &Path,
) {
    let reference_path = Path::new(reference);
    assert!(
        reference_path.is_relative(),
        "plugin manifest {relative_path:?} {context} bevy reference `{reference}` should be repository-relative"
    );
    assert!(
        !reference_path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::CurDir | Component::RootDir | Component::Prefix(_)
            )
        }),
        "plugin manifest {relative_path:?} {context} bevy reference `{reference}` should not contain root, current, parent, or drive-prefix path components"
    );
    assert!(
        reference_path.starts_with(bevy_root),
        "plugin manifest {relative_path:?} {context} bevy reference `{reference}` should stay under dev/bevy"
    );
}

fn visit_capability_status_rows(
    table: &toml::Table,
    relative_path: &Path,
    visit: &mut impl FnMut(&str, String),
) {
    let Some(statuses) = table.get("capability_statuses") else {
        return;
    };
    let statuses = statuses.as_array().unwrap_or_else(|| {
        panic!("plugin manifest {relative_path:?} capability_statuses should be an array")
    });

    for status in statuses {
        let status = status.as_table().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} capability status should be a table")
        });
        let capability =
            non_empty_string_value(status, relative_path, "capability status", "capability");
        visit(capability, format!("capability status `{capability}`"));
    }
}

fn visit_package_dependency_rows(
    table: &toml::Table,
    relative_path: &Path,
    visit: &mut impl FnMut(&str, String),
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
        visit(
            capability,
            format!("top-level dependency `{dependency_id}` capability"),
        );
    }
}

fn visit_optional_feature_dependency_rows(
    feature: &toml::Table,
    relative_path: &Path,
    feature_context: &str,
    visit: &mut impl FnMut(&str, String),
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
        visit(
            capability,
            format!("{feature_context} dependency `{dependency_plugin}` capability"),
        );
    }
}

fn visit_asset_importer_required_capabilities(
    table: &toml::Table,
    relative_path: &Path,
    visit: &mut impl FnMut(&str, String),
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
            visit(capability, importer_context.clone());
        }
    }
}

fn visit_option_required_capabilities(
    table: &toml::Table,
    relative_path: &Path,
    visit: &mut impl FnMut(&str, String),
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
            visit(capability, format!("plugin option `{key}` capability"));
        }
    }
}

fn assert_capability_namespace(relative_path: &Path, context: &str, capability: &str) {
    assert_eq!(
        capability.trim(),
        capability,
        "plugin manifest {relative_path:?} {context} capability `{capability}` should not have leading or trailing whitespace"
    );

    let segments: Vec<_> = capability.split('.').collect();
    assert!(
        segments.len() >= 2,
        "plugin manifest {relative_path:?} {context} capability `{capability}` should use at least two dot-separated namespace segments"
    );

    for segment in segments {
        assert!(
            !segment.is_empty(),
            "plugin manifest {relative_path:?} {context} capability `{capability}` should not contain empty namespace segments"
        );
        assert!(
            segment
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_'),
            "plugin manifest {relative_path:?} {context} capability `{capability}` segment `{segment}` should use lowercase ASCII letters, digits, or underscores"
        );
    }
}

fn assert_unique_capability_owner(
    capability_owners: &mut BTreeMap<String, String>,
    capability: &str,
    context: String,
) {
    if let Some(previous_context) =
        capability_owners.insert(capability.to_string(), context.clone())
    {
        panic!(
            "static plugin capability `{capability}` should have one package or optional-feature owner; first declared by {previous_context}, repeated by {context}"
        );
    }
}
