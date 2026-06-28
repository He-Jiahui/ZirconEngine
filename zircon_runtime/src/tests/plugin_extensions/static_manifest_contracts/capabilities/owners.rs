use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::super::{
    for_each_feature_extension, for_each_optional_feature, for_each_static_plugin_manifest,
    non_empty_string_array_values, non_empty_string_value, visit_module_rows,
};
use super::traversal::visit_capability_status_rows;

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
        let allows_asset_importer_domain_capabilities =
            table.get("category").and_then(toml::Value::as_str) == Some("asset_importer");

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
                    allows_asset_importer_domain_capabilities,
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
                        false,
                    );
                },
            );
        });

        for_each_feature_extension(table, relative_path, &mut |feature, feature_context| {
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
                        false,
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
    allows_asset_importer_domain_capabilities: bool,
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
            allows_asset_importer_domain_capabilities,
        );
    }
}

fn assert_capability_mentions_owner_namespace(
    relative_path: &Path,
    context: &str,
    capability: &str,
    owner_namespace: &str,
    allows_asset_importer_domain_capabilities: bool,
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
    let contains_editor_extension_alias =
        owner_namespace
            .strip_prefix("editor_")
            .is_some_and(|editor_alias| {
                let editor_extension_prefix = format!("editor.extension.{editor_alias}");
                capability == editor_extension_prefix
                    || capability.starts_with(&format!("{editor_extension_prefix}."))
            });
    let contains_feature_alias = owner_namespace == "net.content_download"
        && capability == "runtime.feature.net.cdn_download";
    let contains_script_backend_alias =
        owner_namespace
            .strip_suffix("_language")
            .is_some_and(|language_alias| {
                capability.starts_with(&format!("runtime.script.backend.{language_alias}_"))
            });

    assert!(
        contains_owner_segments
            || contains_owner_prefixed_segment
            || contains_editor_extension_alias
            || contains_feature_alias
            || contains_script_backend_alias
            || (allows_asset_importer_domain_capabilities
                && capability.starts_with("runtime.asset.importer.")),
        "plugin manifest {relative_path:?} {context} capability `{capability}` should stay under owner namespace `{owner_namespace}`"
    );
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
    for_each_feature_extension(table, relative_path, &mut |feature, feature_context| {
        for capability in
            non_empty_string_array_values(feature, relative_path, feature_context, "capabilities")
        {
            capabilities.insert(capability.to_string());
        }
    });

    capabilities
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
