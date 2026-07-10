use std::collections::{HashMap, HashSet};

use crate::plugin::{
    CapabilityStatus, PluginFeatureBundleManifest, PluginModuleManifest, PluginPackageManifest,
};

use super::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};

/// Read-only capability projection derived from concrete plugin registration reports.
#[derive(Clone, Debug, Default)]
pub struct CapabilityView {
    provided: HashSet<String>,
    statuses: HashMap<String, CapabilityStatus>,
}

impl CapabilityView {
    pub fn from_capabilities(capabilities: impl IntoIterator<Item = String>) -> Self {
        Self {
            provided: capabilities.into_iter().collect(),
            statuses: HashMap::new(),
        }
    }

    /// Builds the capability projection from concrete plugin and feature reports only.
    ///
    /// Optional feature declarations remain absent until a provider emits a concrete
    /// `RuntimePluginFeatureRegistrationReport`.
    pub fn from_registration_reports<'a>(
        registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
        feature_registrations: impl IntoIterator<Item = &'a RuntimePluginFeatureRegistrationReport>,
    ) -> Self {
        let mut view = Self::default();
        for registration in registrations {
            view.extend_package_manifest(&registration.package_manifest);
        }
        for feature_registration in feature_registrations {
            view.extend_feature_manifest(&feature_registration.manifest);
        }
        view
    }

    pub fn has(&self, capability: &str) -> bool {
        self.provided.contains(capability)
    }

    pub fn status(&self, capability: &str) -> Option<CapabilityStatus> {
        self.statuses.get(capability).copied()
    }

    pub fn with_status(mut self, capability: impl Into<String>, status: CapabilityStatus) -> Self {
        let capability = capability.into();
        self.provided.insert(capability.clone());
        self.statuses.insert(capability, status);
        self
    }

    fn extend_package_manifest(&mut self, manifest: &PluginPackageManifest) {
        self.extend_capabilities(manifest.capabilities.iter());
        self.extend_module_capabilities(manifest.modules.iter());
        for status in &manifest.capability_statuses {
            self.provided.insert(status.capability.clone());
            self.statuses
                .entry(status.capability.clone())
                .or_insert(status.status);
        }
    }

    fn extend_feature_manifest(&mut self, manifest: &PluginFeatureBundleManifest) {
        self.extend_capabilities(manifest.capabilities.iter());
        self.extend_module_capabilities(manifest.modules.iter());
    }

    fn extend_module_capabilities<'a>(
        &mut self,
        modules: impl IntoIterator<Item = &'a PluginModuleManifest>,
    ) {
        for module in modules {
            self.extend_capabilities(module.capabilities.iter());
        }
    }

    fn extend_capabilities<'a>(&mut self, capabilities: impl IntoIterator<Item = &'a String>) {
        self.provided.extend(capabilities.into_iter().cloned());
    }
}
