use crate::{
    plugin::CapabilityStatusManifest, plugin::ExportPackagingStrategy,
    plugin::PluginFeatureBundleManifest, plugin::PluginMaturity, RuntimePluginId,
    RuntimeTargetMode,
};

mod builder;
mod package_manifest;
mod project_selection;
mod validation;

pub(super) use validation::validate_runtime_plugin_descriptor;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginDescriptor {
    pub package_id: String,
    pub display_name: String,
    pub category: String,
    pub runtime_id: RuntimePluginId,
    pub crate_name: String,
    pub enabled_by_default: bool,
    pub required_by_default: bool,
    pub target_modes: Vec<RuntimeTargetMode>,
    pub capabilities: Vec<String>,
    pub capability_statuses: Vec<CapabilityStatusManifest>,
    pub maturity: PluginMaturity,
    pub optional_features: Vec<PluginFeatureBundleManifest>,
    pub default_packaging: Vec<ExportPackagingStrategy>,
}
