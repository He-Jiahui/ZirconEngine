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

    /// 查询登记中是否出现过该能力；带状态但未列入 provided 的能力同样算作已声明。
    pub fn has(&self, capability: &str) -> bool {
        self.provided.contains(capability) || self.statuses.contains_key(capability)
    }

    pub fn status(&self, capability: &str) -> Option<CapabilityStatus> {
        self.statuses.get(capability).copied()
    }

    pub fn with_status(mut self, capability: impl Into<String>, status: CapabilityStatus) -> Self {
        let capability = capability.into();
        self.provided.remove(&capability);
        self.statuses.insert(capability, status);
        self
    }

    fn extend_package_manifest(&mut self, manifest: &PluginPackageManifest) {
        for status in &manifest.capability_statuses {
            self.provided.remove(status.capability.as_str());
            if !self.statuses.contains_key(status.capability.as_str()) {
                self.statuses
                    .insert(status.capability.clone(), status.status);
            }
        }
        self.extend_capabilities(manifest.capabilities.iter());
        self.extend_module_capabilities(manifest.modules.iter());
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
        for capability in capabilities {
            if !self.statuses.contains_key(capability.as_str()) {
                self.provided.insert(capability.clone());
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/capability_view.rs"]
mod tests;
