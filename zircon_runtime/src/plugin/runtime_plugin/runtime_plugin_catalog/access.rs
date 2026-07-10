use crate::core::CoreError;
use crate::plugin::PluginPackageManifest;

use super::bridge_dependencies::{
    bridge_dependents_for_provider, bridge_disable_blockers_for_provider,
    RuntimePluginBridgeDependent, RuntimePluginBridgeDisableBlocker,
};
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
