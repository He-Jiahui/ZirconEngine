use crate::core::ModuleDescriptor;
use crate::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};
use crate::{
    core::framework::project::ExportPackagingStrategy, plugin::CapabilityStatusManifest,
    plugin::PluginFeatureBundleManifest, plugin::PluginInterfaceManifest, plugin::PluginMaturity,
};

mod access;
mod builder;
mod package_manifest;
mod project_selection;
mod validation;

pub use builder::RuntimePluginDescriptorBuilder;
pub(super) use validation::validate_runtime_plugin_descriptor;

#[derive(Clone, Debug)]
pub struct RuntimePluginDescriptor {
    package_id: String,
    display_name: String,
    category: String,
    runtime_id: RuntimePluginId,
    crate_name: String,
    module_descriptor: ModuleDescriptor,
    enabled_by_default: bool,
    required_by_default: bool,
    target_modes: Vec<RuntimeTargetMode>,
    capabilities: Vec<String>,
    provided_interfaces: Vec<PluginInterfaceManifest>,
    system_sets: Vec<String>,
    system_anchors: Vec<String>,
    capability_statuses: Vec<CapabilityStatusManifest>,
    maturity: PluginMaturity,
    optional_features: Vec<PluginFeatureBundleManifest>,
    default_packaging: Vec<ExportPackagingStrategy>,
}

impl PartialEq for RuntimePluginDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.package_id == other.package_id
            && self.display_name == other.display_name
            && self.category == other.category
            && self.runtime_id == other.runtime_id
            && self.crate_name == other.crate_name
            && module_descriptor_data_eq(&self.module_descriptor, &other.module_descriptor)
            && self.enabled_by_default == other.enabled_by_default
            && self.required_by_default == other.required_by_default
            && self.target_modes == other.target_modes
            && self.capabilities == other.capabilities
            && self.provided_interfaces == other.provided_interfaces
            && self.system_sets == other.system_sets
            && self.system_anchors == other.system_anchors
            && self.capability_statuses == other.capability_statuses
            && self.maturity == other.maturity
            && self.optional_features == other.optional_features
            && self.default_packaging == other.default_packaging
    }
}

impl Eq for RuntimePluginDescriptor {}

fn module_descriptor_data_eq(left: &ModuleDescriptor, right: &ModuleDescriptor) -> bool {
    left.name == right.name
        && left.description == right.description
        && left.init_level == right.init_level
        && left.module_dependencies == right.module_dependencies
}
