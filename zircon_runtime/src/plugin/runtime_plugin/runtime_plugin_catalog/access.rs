use crate::core::framework::project::ProjectPluginFeatureSelection;
use crate::core::CoreError;
use crate::plugin::{PluginFeatureBundleManifest, PluginPackageManifest};

use super::bridge_dependencies::{
    bridge_dependents_for_provider, bridge_disable_blockers_for_provider,
    RuntimePluginBridgeDependent, RuntimePluginBridgeDisableBlocker,
};
use super::feature_definition_collection::feature_definition_map;
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

    pub fn package_manifests(&self) -> Vec<PluginPackageManifest> {
        self.registrations
            .iter()
            .map(|registration| registration.package_manifest.clone())
            .collect()
    }

    pub fn feature_manifest_for_selection(
        &self,
        owner_plugin_id: &str,
        selection: &ProjectPluginFeatureSelection,
    ) -> Option<PluginFeatureBundleManifest> {
        feature_definition_map(&self.registrations, &self.feature_registrations)
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
        bridge_dependents_for_provider(&self.registrations, provider_package_id)
    }

    pub fn strong_bridge_disable_blockers(
        &self,
        provider_package_id: &str,
    ) -> Vec<RuntimePluginBridgeDisableBlocker> {
        bridge_disable_blockers_for_provider(&self.registrations, provider_package_id)
    }
}
