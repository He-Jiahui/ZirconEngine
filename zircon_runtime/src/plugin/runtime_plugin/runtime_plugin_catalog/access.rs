use crate::core::framework::project::ProjectPluginFeatureSelection;
use crate::core::CoreError;
use crate::plugin::{PluginFeatureBundleManifest, PluginPackageManifest};

use super::bridge_dependencies::{RuntimePluginBridgeDependent, RuntimePluginBridgeDisableBlocker};
use super::{
    RuntimePluginCatalog, RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};

impl RuntimePluginCatalog {
    pub fn registrations(&self) -> &[RuntimePluginRegistrationReport] {
        &self.registrations
    }

    pub fn feature_registrations(&self) -> &[RuntimePluginFeatureRegistrationReport] {
        &self.feature_registrations
    }

    pub fn package_manifests(&self) -> impl ExactSizeIterator<Item = &PluginPackageManifest> + '_ {
        self.registrations
            .iter()
            .map(|registration| &registration.package_manifest)
    }

    pub fn feature_manifest_for_selection(
        &self,
        owner_plugin_id: &str,
        selection: &ProjectPluginFeatureSelection,
    ) -> Option<PluginFeatureBundleManifest> {
        self.projection
            .feature_definitions()
            .definition_for_selection(owner_plugin_id, selection)
            .map(|definition| {
                let mut manifest = definition.manifest.clone();
                if manifest.provider_package_id.is_none() {
                    manifest.provider_package_id = definition
                        .external_provider_for_owner()
                        .map(ToOwned::to_owned);
                }
                manifest
            })
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub fn module_order_error(&self) -> Option<&CoreError> {
        self.module_order_error.as_deref()
    }

    pub fn strong_bridge_dependents(
        &self,
        provider_package_id: &str,
    ) -> Vec<RuntimePluginBridgeDependent> {
        self.projection
            .bridge_dependents_for_provider(provider_package_id)
            .to_vec()
    }

    pub fn strong_bridge_disable_blockers(
        &self,
        provider_package_id: &str,
    ) -> Vec<RuntimePluginBridgeDisableBlocker> {
        self.projection
            .bridge_dependents_for_provider(provider_package_id)
            .iter()
            .cloned()
            .map(|dependent| RuntimePluginBridgeDisableBlocker {
                provider_package_id: provider_package_id.to_string(),
                dependent_package_id: dependent.package_id,
                interface_ids: dependent.interface_ids,
            })
            .collect()
    }

    pub fn projection_metrics(&self) -> super::RuntimePluginCatalogProjectionMetrics {
        self.projection.metrics()
    }

    #[cfg(test)]
    pub(super) fn projection_stats(
        &self,
    ) -> super::derived_projection::RuntimePluginCatalogProjectionStats {
        self.projection.stats()
    }
}
