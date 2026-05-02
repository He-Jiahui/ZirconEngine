use crate::{
    plugin::ExportPackagingStrategy, plugin::PluginFeatureBundleManifest, RuntimePluginId, RuntimeTargetMode,
};

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
    pub optional_features: Vec<PluginFeatureBundleManifest>,
    pub default_packaging: Vec<ExportPackagingStrategy>,
}
