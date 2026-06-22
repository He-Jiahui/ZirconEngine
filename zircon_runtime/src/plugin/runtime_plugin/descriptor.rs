use crate::builtin::{RuntimePluginId, RuntimeTargetMode};
use crate::{
    plugin::CapabilityStatusManifest, plugin::ExportPackagingStrategy,
    plugin::PluginFeatureBundleManifest, plugin::PluginMaturity,
};

mod access;
mod builder;
mod package_manifest;
mod project_selection;
mod validation;

pub use builder::RuntimePluginDescriptorBuilder;
pub(super) use validation::validate_runtime_plugin_descriptor;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginDescriptor {
    package_id: String,
    display_name: String,
    category: String,
    runtime_id: RuntimePluginId,
    crate_name: String,
    enabled_by_default: bool,
    required_by_default: bool,
    target_modes: Vec<RuntimeTargetMode>,
    capabilities: Vec<String>,
    system_sets: Vec<String>,
    system_anchors: Vec<String>,
    capability_statuses: Vec<CapabilityStatusManifest>,
    maturity: PluginMaturity,
    optional_features: Vec<PluginFeatureBundleManifest>,
    default_packaging: Vec<ExportPackagingStrategy>,
}
