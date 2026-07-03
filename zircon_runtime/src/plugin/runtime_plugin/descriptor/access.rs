use crate::builtin::{RuntimePluginId, RuntimeTargetMode};
use crate::core::ModuleDescriptor;
use crate::{
    plugin::CapabilityStatusManifest, plugin::ExportPackagingStrategy,
    plugin::PluginFeatureBundleManifest, plugin::PluginInterfaceManifest, plugin::PluginMaturity,
};

use super::RuntimePluginDescriptor;

impl RuntimePluginDescriptor {
    pub fn package_id(&self) -> &str {
        &self.package_id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn category(&self) -> &str {
        &self.category
    }

    pub fn runtime_id(&self) -> RuntimePluginId {
        self.runtime_id
    }

    pub fn crate_name(&self) -> &str {
        &self.crate_name
    }

    pub fn module_descriptor(&self) -> &ModuleDescriptor {
        &self.module_descriptor
    }

    pub fn enabled_by_default(&self) -> bool {
        self.enabled_by_default
    }

    pub fn required_by_default(&self) -> bool {
        self.required_by_default
    }

    pub fn target_modes(&self) -> &[RuntimeTargetMode] {
        &self.target_modes
    }

    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }

    pub fn provided_interfaces(&self) -> &[PluginInterfaceManifest] {
        &self.provided_interfaces
    }

    pub fn system_sets(&self) -> &[String] {
        &self.system_sets
    }

    pub fn system_anchors(&self) -> &[String] {
        &self.system_anchors
    }

    pub fn capability_statuses(&self) -> &[CapabilityStatusManifest] {
        &self.capability_statuses
    }

    pub fn maturity(&self) -> PluginMaturity {
        self.maturity
    }

    pub fn optional_features(&self) -> &[PluginFeatureBundleManifest] {
        &self.optional_features
    }

    pub fn default_packaging(&self) -> &[ExportPackagingStrategy] {
        &self.default_packaging
    }
}
