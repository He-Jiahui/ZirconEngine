use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

mod asset_importers;
mod capabilities;
mod dependencies;
mod event_catalogs;
mod manifest_schema;
mod modules;
mod options;
mod package_coordinates;
mod package_identity;
mod package_layout;
mod package_metadata;
mod package_versions;

#[test]
fn plugin_tomls_declare_public_package_metadata() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for field_name in [
            "id",
            "version",
            "sdk_api_version",
            "display_name",
            "category",
            "description",
            "maturity",
        ] {
            assert_non_empty_string(table, relative_path, "top-level", field_name);
        }

        assert_non_empty_string_array(table, relative_path, "top-level", "supported_targets");
        assert_non_empty_string_array(table, relative_path, "top-level", "capabilities");
    });
}

#[test]
fn plugin_tomls_declare_unique_package_ids() {
    let mut package_ids = BTreeMap::new();

    for_each_static_plugin_manifest(|relative_path, table| {
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");
        assert_unique_static_identity(
            &mut package_ids,
            package_id,
            format!("top-level package in {}", relative_path.display()),
        );
    });
}

#[test]
fn plugin_tomls_declare_unique_optional_feature_ids() {
    let mut static_ids = BTreeMap::new();

    for_each_static_plugin_manifest(|relative_path, table| {
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");
        assert_unique_static_identity(
            &mut static_ids,
            package_id,
            format!("top-level package in {}", relative_path.display()),
        );

        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            let feature_id = non_empty_string_value(feature, relative_path, feature_context, "id");
            assert_unique_static_identity(
                &mut static_ids,
                feature_id,
                format!("{feature_context} in {}", relative_path.display()),
            );
        });
    });
}

#[test]
fn plugin_tomls_declare_known_package_classification() {
    for_each_static_plugin_manifest(|relative_path, table| {
        assert_known_package_category(table, relative_path);
        assert_known_plugin_maturity(table, relative_path);
    });
}

#[test]
fn plugin_tomls_declare_known_runtime_targets() {
    for_each_static_plugin_manifest(|relative_path, table| {
        assert_known_runtime_targets(table, relative_path, "top-level", "supported_targets");

        for_each_module_row(table, relative_path, &mut |module, module_context| {
            let module_name = module
                .get("name")
                .and_then(toml::Value::as_str)
                .unwrap_or("<unknown>");
            assert_known_runtime_targets(
                module,
                relative_path,
                &format!("{module_context} module `{module_name}`"),
                "target_modes",
            );
        });
    });
}

#[test]
fn plugin_tomls_declare_unique_string_array_entries() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for field_name in ["supported_targets", "capabilities", "default_packaging"] {
            assert_unique_string_array_entries(table, relative_path, "top-level", field_name);
        }

        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            for field_name in ["capabilities", "default_packaging"] {
                assert_unique_string_array_entries(
                    feature,
                    relative_path,
                    feature_context,
                    field_name,
                );
            }
        });

        for_each_module_row(table, relative_path, &mut |module, module_context| {
            let module_name = module
                .get("name")
                .and_then(toml::Value::as_str)
                .unwrap_or("<unknown>");
            let module_context = format!("{module_context} module `{module_name}`");
            for field_name in ["capabilities", "target_modes"] {
                assert_unique_string_array_entries(
                    module,
                    relative_path,
                    &module_context,
                    field_name,
                );
            }
        });
    });
}

#[test]
fn plugin_tomls_declare_capability_status_rows() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let Some(statuses) = table.get("capability_statuses") else {
            return;
        };
        let statuses = statuses.as_array().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} capability_statuses should be an array")
        });
        assert!(
            !statuses.is_empty(),
            "plugin manifest {relative_path:?} capability_statuses should not be empty when declared"
        );

        let mut capability_statuses = BTreeMap::new();
        for status in statuses {
            let status = status.as_table().unwrap_or_else(|| {
                panic!("plugin manifest {relative_path:?} capability status should be a table")
            });
            let capability =
                non_empty_string_value(status, relative_path, "capability status", "capability");
            let status_context = format!("capability status `{capability}`");
            if let Some(previous_context) =
                capability_statuses.insert(capability.to_string(), status_context.clone())
            {
                panic!(
                    "plugin manifest {relative_path:?} capability status `{capability}` should be unique; first declared by {previous_context}, repeated by {status_context}"
                );
            }

            assert_known_capability_status(status, relative_path, &status_context);
            if status.get("target_modes").is_some() {
                assert_known_runtime_targets(
                    status,
                    relative_path,
                    &status_context,
                    "target_modes",
                );
                assert_unique_string_array_entries(
                    status,
                    relative_path,
                    &status_context,
                    "target_modes",
                );
            }
            assert_optional_unique_string_array_entries(
                status,
                relative_path,
                &status_context,
                "bevy_references",
            );
            if status.get("note").is_some() {
                assert_non_empty_string(status, relative_path, &status_context, "note");
            }
        }
    });
}

#[test]
fn plugin_tomls_declare_default_packaging_options() {
    for_each_static_plugin_manifest(|relative_path, table| {
        assert_non_empty_string_array(table, relative_path, "top-level", "default_packaging");
    });
}

#[test]
fn plugin_tomls_declare_known_default_packaging_strategies() {
    for_each_static_plugin_manifest(|relative_path, table| {
        assert_known_default_packaging_strategies(table, relative_path, "top-level");

        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            assert_known_default_packaging_strategies(feature, relative_path, feature_context);
        });
    });
}

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
            non_empty_string_value(
                dependency,
                relative_path,
                &format!("top-level dependency `{dependency_id}`"),
                "capability",
            );
            bool_value(
                dependency,
                relative_path,
                &format!("top-level dependency `{dependency_id}`"),
                "required",
            );
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
            let dependency_capability = non_empty_string_value(
                dependency,
                relative_path,
                &format!("top-level dependency `{dependency_id}`"),
                "capability",
            );
            assert_unique_dependency_row(
                &mut dependency_rows,
                dependency_id,
                dependency_capability,
                format!(
                    "top-level dependency `{dependency_id}` in {}",
                    relative_path.display()
                ),
            );
        }
    });
}

#[test]
fn plugin_tomls_declare_optional_feature_metadata() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");

        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            assert_eq!(
                feature
                    .get("owner_plugin_id")
                    .and_then(toml::Value::as_str),
                Some(package_id),
                "plugin manifest {relative_path:?} {feature_context} should declare owner_plugin_id matching package id `{package_id}`"
            );
            for field_name in ["id", "display_name"] {
                assert_non_empty_string(feature, relative_path, feature_context, field_name);
            }
            for field_name in ["capabilities", "default_packaging"] {
                assert_non_empty_string_array(feature, relative_path, feature_context, field_name);
            }
            assert!(
                feature.get("enabled_by_default").and_then(toml::Value::as_bool).is_some(),
                "plugin manifest {relative_path:?} {feature_context} should declare boolean `enabled_by_default`"
            );
        });
    });
}

#[test]
fn plugin_tomls_declare_optional_feature_ids_under_owner_namespace() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");
        let expected_prefix = format!("{package_id}.");

        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            let feature_id = non_empty_string_value(feature, relative_path, feature_context, "id");
            assert!(
                feature_id.starts_with(&expected_prefix),
                "plugin manifest {relative_path:?} {feature_context} id `{feature_id}` should stay under owner namespace `{expected_prefix}`"
            );
        });
    });
}

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

#[test]
fn plugin_tomls_declare_module_identity() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for_each_module_row(table, relative_path, &mut |module, module_context| {
            let module_name = non_empty_string_value(module, relative_path, module_context, "name");
            let module_context = format!("{module_context} module `{module_name}`");
            let module_kind =
                non_empty_string_value(module, relative_path, &module_context, "kind");
            assert!(
                matches!(module_kind, "runtime" | "editor" | "native" | "vm"),
                "plugin manifest {relative_path:?} {module_context} should declare a known module kind, got `{module_kind}`"
            );
            assert_non_empty_string(module, relative_path, &module_context, "crate_name");
        });
    });
}

#[test]
fn plugin_tomls_declare_unique_module_names() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let mut module_names = BTreeMap::new();

        for_each_module_row(table, relative_path, &mut |module, module_context| {
            let module_name = non_empty_string_value(module, relative_path, module_context, "name");
            if let Some(previous_context) =
                module_names.insert(module_name.to_string(), module_context.to_string())
            {
                panic!(
                    "plugin manifest {relative_path:?} module name `{module_name}` should be unique across package and optional-feature module rows; first declared in {previous_context}, repeated in {module_context}"
                );
            }
        });
    });
}

#[test]
fn plugin_tomls_declare_module_capabilities() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for_each_module_row(table, relative_path, &mut |module, module_context| {
            assert_module_non_empty_string_array(
                module,
                relative_path,
                module_context,
                "capabilities",
            );
        });
    });
}

#[test]
fn plugin_tomls_declare_module_target_modes() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for_each_module_row(table, relative_path, &mut |module, module_context| {
            assert_module_non_empty_string_array(
                module,
                relative_path,
                module_context,
                "target_modes",
            );
        });
    });
}

fn for_each_static_plugin_manifest(mut visit: impl FnMut(&Path, &toml::Table)) {
    let plugins_root = plugins_workspace_root();
    let mut manifest_paths = Vec::new();
    collect_plugin_manifest_paths(&plugins_root, &mut manifest_paths);
    manifest_paths.sort();

    assert!(
        !manifest_paths.is_empty(),
        "zircon_plugins workspace should contain package manifests"
    );

    for manifest_path in manifest_paths {
        let source = fs::read_to_string(&manifest_path)
            .unwrap_or_else(|error| panic!("plugin manifest {manifest_path:?}: {error}"));
        let value: toml::Value = toml::from_str(&source)
            .unwrap_or_else(|error| panic!("plugin manifest {manifest_path:?}: {error}"));
        let table = value
            .as_table()
            .unwrap_or_else(|| panic!("plugin manifest {manifest_path:?} should be a table"));
        let relative_path = manifest_path
            .strip_prefix(&plugins_root)
            .unwrap_or(&manifest_path);

        visit(relative_path, table);
    }
}

fn for_each_module_row(
    table: &toml::Table,
    relative_path: &Path,
    visit: &mut impl FnMut(&toml::Table, &str),
) {
    visit_module_rows(table.get("modules"), relative_path, "package", visit);

    if let Some(optional_features) = table
        .get("optional_features")
        .and_then(toml::Value::as_array)
    {
        for feature in optional_features {
            let feature_table = feature.as_table().unwrap_or_else(|| {
                panic!("plugin manifest {relative_path:?} optional feature should be a table")
            });
            let feature_id = feature_table
                .get("id")
                .and_then(toml::Value::as_str)
                .unwrap_or("<unknown>");
            let context = format!("optional feature `{feature_id}`");
            visit_module_rows(feature_table.get("modules"), relative_path, &context, visit);
        }
    }
}

fn for_each_optional_feature(
    table: &toml::Table,
    relative_path: &Path,
    visit: &mut impl FnMut(&toml::Table, &str),
) {
    let Some(optional_features) = table
        .get("optional_features")
        .and_then(toml::Value::as_array)
    else {
        return;
    };

    for feature in optional_features {
        let feature_table = feature.as_table().unwrap_or_else(|| {
            panic!("plugin manifest {relative_path:?} optional feature should be a table")
        });
        let feature_id = feature_table
            .get("id")
            .and_then(toml::Value::as_str)
            .unwrap_or("<unknown>");
        let context = format!("optional feature `{feature_id}`");
        visit(feature_table, &context);
    }
}

fn visit_module_rows(
    modules: Option<&toml::Value>,
    relative_path: &Path,
    module_context: &str,
    visit: &mut impl FnMut(&toml::Table, &str),
) {
    let Some(modules) = modules else {
        return;
    };
    let modules = modules.as_array().unwrap_or_else(|| {
        panic!("plugin manifest {relative_path:?} {module_context} modules should be an array")
    });

    for module in modules {
        let module = module.as_table().unwrap_or_else(|| {
            panic!(
                "plugin manifest {relative_path:?} {module_context} module row should be a table"
            )
        });
        visit(module, module_context);
    }
}

fn assert_module_non_empty_string_array(
    module: &toml::Table,
    relative_path: &Path,
    module_context: &str,
    field_name: &str,
) {
    let module_name = module
        .get("name")
        .and_then(toml::Value::as_str)
        .unwrap_or("<unknown>");
    assert_non_empty_string_array(
        module,
        relative_path,
        &format!("{module_context} module `{module_name}`"),
        field_name,
    );
}

fn assert_non_empty_string(
    table: &toml::Table,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) {
    non_empty_string_value(table, relative_path, context, field_name);
}

fn assert_non_empty_string_array(
    table: &toml::Table,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) {
    non_empty_string_array_values(table, relative_path, context, field_name);
}

fn assert_known_package_category(table: &toml::Table, relative_path: &Path) {
    let category = non_empty_string_value(table, relative_path, "top-level", "category");
    assert!(
        matches!(
            category,
            "asset_importer" | "authoring" | "diagnostics" | "platform" | "rendering" | "runtime" | "sdk"
        ),
        "plugin manifest {relative_path:?} top-level category `{category}` should be a known package category"
    );
}

fn assert_known_plugin_maturity(table: &toml::Table, relative_path: &Path) {
    let maturity = non_empty_string_value(table, relative_path, "top-level", "maturity");
    assert!(
        matches!(
            maturity,
            "core" | "stable" | "beta" | "experimental" | "externalized" | "stub" | "deprecated"
        ),
        "plugin manifest {relative_path:?} top-level maturity `{maturity}` should be a known plugin maturity"
    );
}

fn assert_known_capability_status(table: &toml::Table, relative_path: &Path, context: &str) {
    let status = non_empty_string_value(table, relative_path, context, "status");
    assert!(
        matches!(
            status,
            "complete" | "partial" | "stub" | "externalized" | "unsupported"
        ),
        "plugin manifest {relative_path:?} {context} status `{status}` should be a known capability status"
    );
}

fn assert_unique_static_identity(
    identities: &mut BTreeMap<String, String>,
    identity: &str,
    context: String,
) {
    if let Some(previous_context) = identities.insert(identity.to_string(), context.clone()) {
        panic!(
            "static plugin identity `{identity}` should be globally unique; first declared by {previous_context}, repeated by {context}"
        );
    }
}

fn assert_unique_dependency_row(
    dependency_rows: &mut BTreeMap<String, String>,
    plugin_id: &str,
    capability: &str,
    context: String,
) {
    let dependency_key = format!("{plugin_id}:{capability}");
    if let Some(previous_context) = dependency_rows.insert(dependency_key.clone(), context.clone())
    {
        panic!(
            "dependency row `{dependency_key}` should be unique; first declared by {previous_context}, repeated by {context}"
        );
    }
}

fn assert_unique_string_array_entries(
    table: &toml::Table,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) {
    let mut entries = BTreeMap::new();
    for (index, entry) in non_empty_string_array_values(table, relative_path, context, field_name)
        .into_iter()
        .enumerate()
    {
        if let Some(previous_index) = entries.insert(entry.to_string(), index) {
            panic!(
                "plugin manifest {relative_path:?} {context} `{field_name}` entry `{entry}` should be unique; first declared at index {previous_index}, repeated at index {index}"
            );
        }
    }
}

fn assert_optional_unique_string_array_entries(
    table: &toml::Table,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) {
    if table.get(field_name).is_some() {
        assert_unique_string_array_entries(table, relative_path, context, field_name);
    }
}

fn assert_known_default_packaging_strategies(
    table: &toml::Table,
    relative_path: &Path,
    context: &str,
) {
    for packaging in
        non_empty_string_array_values(table, relative_path, context, "default_packaging")
    {
        assert!(
            matches!(packaging, "source_template" | "library_embed" | "native_dynamic"),
            "plugin manifest {relative_path:?} {context} default packaging strategy `{packaging}` should be source_template, library_embed, or native_dynamic"
        );
    }
}

fn assert_known_runtime_targets(
    table: &toml::Table,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) {
    for target in non_empty_string_array_values(table, relative_path, context, field_name) {
        assert!(
            matches!(target, "client_runtime" | "server_runtime" | "editor_host"),
            "plugin manifest {relative_path:?} {context} `{field_name}` target `{target}` should be client_runtime, server_runtime, or editor_host"
        );
    }
}

fn non_empty_string_value<'a>(
    table: &'a toml::Table,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) -> &'a str {
    let value = table
        .get(field_name)
        .and_then(toml::Value::as_str)
        .unwrap_or_else(|| {
            panic!(
                "plugin manifest {relative_path:?} {context} should declare non-empty string `{field_name}`"
            )
        });
    assert!(
        !value.is_empty(),
        "plugin manifest {relative_path:?} {context} should declare non-empty string `{field_name}`"
    );
    value
}

fn non_empty_string_array_values<'a>(
    table: &'a toml::Table,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) -> Vec<&'a str> {
    let values = table
        .get(field_name)
        .and_then(toml::Value::as_array)
        .unwrap_or_else(|| {
            panic!(
                "plugin manifest {relative_path:?} {context} should declare non-empty string `{field_name}`"
            )
        });
    assert!(
        !values.is_empty(),
        "plugin manifest {relative_path:?} {context} should declare non-empty string `{field_name}`"
    );
    values
        .iter()
        .map(|value| {
            value.as_str().unwrap_or_else(|| {
                panic!(
                    "plugin manifest {relative_path:?} {context} `{field_name}` entries should be strings"
                )
            })
        })
        .inspect(|value| {
            assert!(
                !value.is_empty(),
                "plugin manifest {relative_path:?} {context} `{field_name}` entries should not be empty"
            );
        })
        .collect()
}

fn bool_value(table: &toml::Table, relative_path: &Path, context: &str, field_name: &str) -> bool {
    table
        .get(field_name)
        .and_then(toml::Value::as_bool)
        .unwrap_or_else(|| {
            panic!(
                "plugin manifest {relative_path:?} {context} should declare boolean `{field_name}`"
            )
        })
}

fn integer_value(
    table: &toml::Table,
    relative_path: &Path,
    context: &str,
    field_name: &str,
) -> i64 {
    table
        .get(field_name)
        .and_then(toml::Value::as_integer)
        .unwrap_or_else(|| {
            panic!(
                "plugin manifest {relative_path:?} {context} should declare integer `{field_name}`"
            )
        })
}

fn plugins_workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("runtime crate should have a repository parent")
        .join("zircon_plugins")
}

fn collect_plugin_manifest_paths(root: &Path, manifest_paths: &mut Vec<PathBuf>) {
    for entry in
        fs::read_dir(root).unwrap_or_else(|error| panic!("read plugin directory {root:?}: {error}"))
    {
        let entry =
            entry.unwrap_or_else(|error| panic!("read plugin directory entry {root:?}: {error}"));
        let path = entry.path();
        if path.is_dir() {
            collect_plugin_manifest_paths(&path, manifest_paths);
        } else if path.file_name().and_then(|name| name.to_str()) == Some("plugin.toml") {
            manifest_paths.push(path);
        }
    }
}
