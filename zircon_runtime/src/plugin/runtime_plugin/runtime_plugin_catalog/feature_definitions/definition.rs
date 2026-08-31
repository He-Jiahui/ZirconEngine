use crate::plugin::PluginFeatureBundleManifest;

use super::key::feature_definition_key;

#[derive(Clone, Debug)]
pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) struct FeatureDefinition {
    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) key: String,
    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) manifest:
        PluginFeatureBundleManifest,
    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) provider_package_id: String,
}

impl FeatureDefinition {
    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn new(
        manifest: PluginFeatureBundleManifest,
        provider_package_id: String,
    ) -> Self {
        let key = Self::key(&manifest.id, &provider_package_id);
        Self::new_with_key(key, manifest, provider_package_id)
    }

    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn key(
        feature_id: &str,
        provider_package_id: &str,
    ) -> String {
        feature_definition_key(feature_id, provider_package_id)
    }

    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn new_with_key(
        key: String,
        manifest: PluginFeatureBundleManifest,
        provider_package_id: String,
    ) -> Self {
        debug_assert_eq!(key, Self::key(&manifest.id, &provider_package_id));
        Self {
            key,
            manifest,
            provider_package_id,
        }
    }

    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn external_provider_for_owner(
        &self,
    ) -> Option<&str> {
        (self.provider_package_id != self.manifest.owner_plugin_id)
            .then_some(self.provider_package_id.as_str())
    }
}
