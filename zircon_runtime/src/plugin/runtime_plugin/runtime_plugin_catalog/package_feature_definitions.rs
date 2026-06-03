use crate::plugin::{PluginPackageKind, PluginPackageManifest};

use super::feature_definitions::FeatureDefinition;

pub(super) fn package_feature_definitions(
    package_manifest: &PluginPackageManifest,
) -> Vec<FeatureDefinition> {
    let mut definitions = Vec::new();
    for feature in &package_manifest.optional_features {
        let provider_package_id = if package_manifest.package_kind
            == PluginPackageKind::FeatureExtension
            || feature.owner_plugin_id != package_manifest.id
        {
            package_manifest.id.clone()
        } else {
            feature.owner_plugin_id.clone()
        };
        definitions.push(FeatureDefinition::new(feature.clone(), provider_package_id));
    }
    for feature in &package_manifest.feature_extensions {
        definitions.push(FeatureDefinition::new(
            feature.clone(),
            package_manifest.id.clone(),
        ));
    }
    definitions
}
