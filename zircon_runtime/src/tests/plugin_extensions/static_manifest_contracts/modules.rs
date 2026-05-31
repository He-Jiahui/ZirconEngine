use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use super::{
    for_each_module_row, for_each_optional_feature, for_each_static_plugin_manifest,
    non_empty_string_array_values, non_empty_string_value, plugins_workspace_root,
    visit_module_rows,
};

struct WorkspaceCrate {
    member_path: String,
}

#[test]
fn plugin_tomls_declare_module_names_under_owner_namespace() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");
        let package_prefix = format!("{package_id}.");

        visit_module_rows(
            table.get("modules"),
            relative_path,
            "package",
            &mut |module, module_context| {
                let module_name =
                    non_empty_string_value(module, relative_path, module_context, "name");
                assert_module_name_prefix(
                    relative_path,
                    module_context,
                    module_name,
                    &package_prefix,
                );
            },
        );

        for_each_optional_feature(table, relative_path, &mut |feature, feature_context| {
            let feature_id = non_empty_string_value(feature, relative_path, feature_context, "id");
            let feature_prefix = format!("{feature_id}.");
            visit_module_rows(
                feature.get("modules"),
                relative_path,
                feature_context,
                &mut |module, module_context| {
                    let module_name =
                        non_empty_string_value(module, relative_path, module_context, "name");
                    assert_module_name_prefix(
                        relative_path,
                        module_context,
                        module_name,
                        &feature_prefix,
                    );
                },
            );
        });
    });
}

#[test]
fn plugin_tomls_declare_module_names_are_dot_namespaced() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for_each_module_row(table, relative_path, &mut |module, module_context| {
            let module_name = non_empty_string_value(module, relative_path, module_context, "name");
            assert_dot_namespaced_module_name(relative_path, module_context, module_name);
        });
    });
}

#[test]
fn plugin_tomls_declare_module_names_match_module_kind() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for_each_module_row(table, relative_path, &mut |module, module_context| {
            let module_name = non_empty_string_value(module, relative_path, module_context, "name");
            let module_kind = non_empty_string_value(module, relative_path, module_context, "kind");
            let expected_suffix = match module_kind {
                "runtime" => ".runtime",
                "editor" => ".editor",
                _ => return,
            };

            assert!(
                module_name.ends_with(expected_suffix),
                "plugin manifest {relative_path:?} {module_context} module name `{module_name}` with kind `{module_kind}` should end with `{expected_suffix}`"
            );
        });
    });
}

#[test]
fn plugin_tomls_declare_module_crates_in_plugin_workspace() {
    let workspace_crates = plugin_workspace_crates();

    for_each_static_plugin_manifest(|relative_path, table| {
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");

        for_each_module_row(table, relative_path, &mut |module, module_context| {
            let module_name = non_empty_string_value(module, relative_path, module_context, "name");
            let module_context = format!("{module_context} module `{module_name}`");
            let crate_name =
                non_empty_string_value(module, relative_path, &module_context, "crate_name");
            assert_crate_name_shape(relative_path, &module_context, crate_name);

            let workspace_crate = workspace_crates.get(crate_name).unwrap_or_else(|| {
                panic!(
                    "plugin manifest {relative_path:?} {module_context} crate_name `{crate_name}` should match a package.name declared by zircon_plugins/Cargo.toml workspace members"
                )
            });
            assert_crate_stays_under_declaring_package(
                relative_path,
                &module_context,
                crate_name,
                package_id,
                &workspace_crate.member_path,
            );
        });
    });
}

#[test]
fn plugin_tomls_declare_module_targets_within_package_targets() {
    for_each_static_plugin_manifest(|relative_path, table| {
        let package_targets =
            non_empty_string_array_values(table, relative_path, "top-level", "supported_targets");

        for_each_module_row(table, relative_path, &mut |module, module_context| {
            let module_name = non_empty_string_value(module, relative_path, module_context, "name");
            let module_context = format!("{module_context} module `{module_name}`");
            for target_mode in non_empty_string_array_values(
                module,
                relative_path,
                &module_context,
                "target_modes",
            ) {
                assert!(
                    package_targets.contains(&target_mode),
                    "plugin manifest {relative_path:?} {module_context} target mode `{target_mode}` should be covered by package supported_targets"
                );
            }
        });
    });
}

#[test]
fn plugin_tomls_declare_editor_modules_target_editor_host_only() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for_each_module_row(table, relative_path, &mut |module, module_context| {
            let module_name = non_empty_string_value(module, relative_path, module_context, "name");
            let module_context = format!("{module_context} module `{module_name}`");
            let module_kind =
                non_empty_string_value(module, relative_path, &module_context, "kind");

            if module_kind != "editor" {
                return;
            }

            for target_mode in non_empty_string_array_values(
                module,
                relative_path,
                &module_context,
                "target_modes",
            ) {
                assert_eq!(
                    target_mode, "editor_host",
                    "plugin manifest {relative_path:?} {module_context} is an editor module and should only target editor_host, got `{target_mode}`"
                );
            }
        });
    });
}

#[test]
fn plugin_tomls_declare_module_capabilities_match_module_kind() {
    for_each_static_plugin_manifest(|relative_path, table| {
        for_each_module_row(table, relative_path, &mut |module, module_context| {
            let module_name = non_empty_string_value(module, relative_path, module_context, "name");
            let module_context = format!("{module_context} module `{module_name}`");
            let module_kind =
                non_empty_string_value(module, relative_path, &module_context, "kind");
            let expected_prefix = match module_kind {
                "runtime" => "runtime.",
                "editor" => "editor.",
                _ => return,
            };

            for capability in non_empty_string_array_values(
                module,
                relative_path,
                &module_context,
                "capabilities",
            ) {
                assert!(
                    capability.starts_with(expected_prefix),
                    "plugin manifest {relative_path:?} {module_context} kind `{module_kind}` capability `{capability}` should start with `{expected_prefix}`"
                );
            }
        });
    });
}

fn plugin_workspace_crates() -> BTreeMap<String, WorkspaceCrate> {
    let plugins_root = plugins_workspace_root();
    let workspace_manifest_path = plugins_root.join("Cargo.toml");
    let source = fs::read_to_string(&workspace_manifest_path).unwrap_or_else(|error| {
        panic!("read plugin workspace manifest {workspace_manifest_path:?}: {error}")
    });
    let value: toml::Value = toml::from_str(&source).unwrap_or_else(|error| {
        panic!("parse plugin workspace manifest {workspace_manifest_path:?}: {error}")
    });
    let members = value
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(toml::Value::as_array)
        .unwrap_or_else(|| {
            panic!("plugin workspace manifest {workspace_manifest_path:?} should declare workspace.members")
        });

    let mut workspace_crates = BTreeMap::new();
    for member in members {
        let member_path = member.as_str().unwrap_or_else(|| {
            panic!(
                "plugin workspace manifest {workspace_manifest_path:?} members should be strings"
            )
        });
        assert_workspace_member_path(&workspace_manifest_path, member_path);

        let package_manifest_path = plugins_root.join(member_path).join("Cargo.toml");
        let package_source = fs::read_to_string(&package_manifest_path).unwrap_or_else(|error| {
            panic!("read plugin workspace package manifest {package_manifest_path:?}: {error}")
        });
        let package_value: toml::Value = toml::from_str(&package_source).unwrap_or_else(|error| {
            panic!("parse plugin workspace package manifest {package_manifest_path:?}: {error}")
        });
        let package_name = package_value
            .get("package")
            .and_then(|package| package.get("name"))
            .and_then(toml::Value::as_str)
            .unwrap_or_else(|| {
                panic!(
                    "plugin workspace member {package_manifest_path:?} should declare package.name"
                )
            });

        if let Some(previous) = workspace_crates.insert(
            package_name.to_string(),
            WorkspaceCrate {
                member_path: member_path.to_string(),
            },
        ) {
            panic!(
                "plugin workspace package name `{package_name}` should be unique; first declared at `{}`, repeated at `{member_path}`",
                previous.member_path
            );
        }
    }

    workspace_crates
}

fn assert_module_name_prefix(
    relative_path: &Path,
    context: &str,
    module_name: &str,
    expected_prefix: &str,
) {
    assert!(
        module_name.starts_with(expected_prefix),
        "plugin manifest {relative_path:?} {context} module name `{module_name}` should stay under namespace `{expected_prefix}`"
    );
}

fn assert_dot_namespaced_module_name(relative_path: &Path, context: &str, module_name: &str) {
    assert_eq!(
        module_name.trim(),
        module_name,
        "plugin manifest {relative_path:?} {context} module name `{module_name}` should not have leading or trailing whitespace"
    );

    let segments: Vec<_> = module_name.split('.').collect();
    assert!(
        segments.len() >= 2,
        "plugin manifest {relative_path:?} {context} module name `{module_name}` should use package.module dot namespace form"
    );

    for segment in segments {
        assert!(
            !segment.is_empty(),
            "plugin manifest {relative_path:?} {context} module name `{module_name}` should not contain empty namespace segments"
        );
        assert!(
            segment
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'),
            "plugin manifest {relative_path:?} {context} module name `{module_name}` should contain only lowercase ASCII letters, digits, underscores, and dots"
        );
    }
}

fn assert_crate_name_shape(relative_path: &Path, context: &str, crate_name: &str) {
    assert_eq!(
        crate_name.trim(),
        crate_name,
        "plugin manifest {relative_path:?} {context} crate_name `{crate_name}` should not have leading or trailing whitespace"
    );
    assert!(
        crate_name.starts_with("zircon_plugin_"),
        "plugin manifest {relative_path:?} {context} crate_name `{crate_name}` should use the `zircon_plugin_` prefix"
    );
    assert!(
        crate_name
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_'),
        "plugin manifest {relative_path:?} {context} crate_name `{crate_name}` should use lowercase ASCII letters, digits, or underscores"
    );
}

fn assert_crate_stays_under_declaring_package(
    relative_path: &Path,
    context: &str,
    crate_name: &str,
    package_id: &str,
    member_path: &str,
) {
    let package_root = format!("{package_id}/");
    assert!(
        member_path == package_id || member_path.starts_with(&package_root),
        "plugin manifest {relative_path:?} {context} crate_name `{crate_name}` resolves to workspace member `{member_path}`, which should stay under declaring package root `{package_id}`"
    );
}

fn assert_workspace_member_path(workspace_manifest_path: &Path, member_path: &str) {
    assert!(
        !member_path.trim().is_empty(),
        "plugin workspace manifest {workspace_manifest_path:?} members should not contain empty paths"
    );
    assert_eq!(
        member_path.trim(),
        member_path,
        "plugin workspace manifest {workspace_manifest_path:?} member path `{member_path}` should not have leading or trailing whitespace"
    );
    assert!(
        !Path::new(member_path).is_absolute(),
        "plugin workspace manifest {workspace_manifest_path:?} member path `{member_path}` should be relative"
    );
    assert!(
        !member_path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == ".."),
        "plugin workspace manifest {workspace_manifest_path:?} member path `{member_path}` should use normalized relative segments"
    );
    assert!(
        !member_path.contains('\\'),
        "plugin workspace manifest {workspace_manifest_path:?} member path `{member_path}` should use forward slashes"
    );
}
