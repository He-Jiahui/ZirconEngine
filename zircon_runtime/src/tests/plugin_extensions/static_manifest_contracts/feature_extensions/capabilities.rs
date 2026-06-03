use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::{
    for_each_static_plugin_manifest, non_empty_string_array_values, non_empty_string_value,
    optional_table_array,
};

pub(super) struct StaticPackageCapabilities {
    pub(super) package_capabilities: BTreeSet<String>,
    feature_capabilities: BTreeSet<String>,
}

impl StaticPackageCapabilities {
    pub(super) fn contains_declared(&self, capability: &str) -> bool {
        self.package_capabilities.contains(capability)
            || self.feature_capabilities.contains(capability)
    }
}

pub(super) fn static_package_capabilities() -> BTreeMap<String, StaticPackageCapabilities> {
    let mut package_capabilities = BTreeMap::new();

    for_each_static_plugin_manifest(|relative_path, table| {
        let package_id = non_empty_string_value(table, relative_path, "top-level", "id");
        let mut package_rows = BTreeSet::new();
        let mut feature_rows = BTreeSet::new();

        for capability in
            non_empty_string_array_values(table, relative_path, "top-level", "capabilities")
        {
            package_rows.insert(capability.to_string());
        }
        collect_feature_bundle_capabilities(
            table,
            relative_path,
            "optional_features",
            "optional feature",
            &mut feature_rows,
        );
        collect_feature_bundle_capabilities(
            table,
            relative_path,
            "feature_extensions",
            "feature extension",
            &mut feature_rows,
        );

        package_capabilities.insert(
            package_id.to_string(),
            StaticPackageCapabilities {
                package_capabilities: package_rows,
                feature_capabilities: feature_rows,
            },
        );
    });

    package_capabilities
}

fn collect_feature_bundle_capabilities(
    table: &toml::Table,
    relative_path: &Path,
    field_name: &str,
    row_context: &str,
    capabilities: &mut BTreeSet<String>,
) {
    let Some(features) = optional_table_array(table, relative_path, "top-level", field_name) else {
        return;
    };

    for feature in features {
        let feature_id = feature
            .get("id")
            .and_then(toml::Value::as_str)
            .unwrap_or("<unknown>");
        let feature_context = format!("{row_context} `{feature_id}`");
        for capability in
            non_empty_string_array_values(feature, relative_path, &feature_context, "capabilities")
        {
            capabilities.insert(capability.to_string());
        }
    }
}
