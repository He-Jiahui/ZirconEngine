use std::collections::{BTreeMap, BTreeSet};

use super::super::{
    for_each_feature_extension, for_each_optional_feature, for_each_static_plugin_manifest,
    non_empty_string_array_values, non_empty_string_value,
};

pub(super) struct StaticPackageCapabilities {
    pub(super) capabilities: BTreeSet<String>,
}

pub(super) fn static_package_capabilities() -> BTreeMap<String, StaticPackageCapabilities> {
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
        for_each_feature_extension(table, relative_path, &mut |feature, feature_context| {
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

pub(super) fn static_declared_capabilities() -> BTreeSet<String> {
    let mut capabilities = BTreeSet::new();

    for package in static_package_capabilities().into_values() {
        capabilities.extend(package.capabilities);
    }

    capabilities
}
