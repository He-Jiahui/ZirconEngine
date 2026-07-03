use crate::builtin::{RuntimePluginId, RuntimeTargetMode};
use crate::core::{InitLevel, ModuleDependencySpec, ModuleDescriptor};
use crate::{
    plugin::CapabilityStatusManifest, plugin::ExportPackagingStrategy,
    plugin::PluginFeatureBundleManifest, plugin::PluginInterfaceManifest, plugin::PluginMaturity,
};

use super::super::RuntimePluginDescriptor;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginDescriptorBuilder {
    descriptor: RuntimePluginDescriptor,
}

impl RuntimePluginDescriptor {
    pub fn builder(
        package_id: impl Into<String>,
        display_name: impl Into<String>,
        runtime_id: RuntimePluginId,
        crate_name: impl Into<String>,
    ) -> RuntimePluginDescriptorBuilder {
        RuntimePluginDescriptorBuilder::new(package_id, display_name, runtime_id, crate_name)
    }
}

impl RuntimePluginDescriptorBuilder {
    pub fn new(
        package_id: impl Into<String>,
        display_name: impl Into<String>,
        runtime_id: RuntimePluginId,
        crate_name: impl Into<String>,
    ) -> Self {
        let package_id = package_id.into();
        let display_name = display_name.into();
        let crate_name = crate_name.into();
        let module_descriptor = ModuleDescriptor::new(
            format!("{package_id}.runtime"),
            format!("Runtime plugin module for {display_name}"),
        )
        .with_init_level(InitLevel::Post);

        Self {
            descriptor: RuntimePluginDescriptor {
                package_id,
                display_name,
                category: "runtime".to_string(),
                runtime_id,
                crate_name,
                module_descriptor,
                enabled_by_default: true,
                required_by_default: false,
                target_modes: Vec::new(),
                capabilities: Vec::new(),
                provided_interfaces: Vec::new(),
                system_sets: Vec::new(),
                system_anchors: Vec::new(),
                capability_statuses: Vec::new(),
                maturity: PluginMaturity::default(),
                optional_features: Vec::new(),
                default_packaging: vec![
                    ExportPackagingStrategy::SourceTemplate,
                    ExportPackagingStrategy::LibraryEmbed,
                ],
            },
        }
    }

    pub fn with_module_descriptor(mut self, descriptor: ModuleDescriptor) -> Self {
        self.descriptor.module_descriptor = descriptor;
        self
    }

    pub fn with_init_level(mut self, init_level: InitLevel) -> Self {
        self.descriptor.module_descriptor.init_level = init_level;
        self
    }

    pub fn with_module_dependency(mut self, dependency: ModuleDependencySpec) -> Self {
        self.descriptor
            .module_descriptor
            .module_dependencies
            .push(dependency);
        self
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.descriptor.category = category.into();
        self
    }

    pub fn with_required_by_default(mut self, required: bool) -> Self {
        self.descriptor.required_by_default = required;
        self
    }

    pub fn with_enabled_by_default(mut self, enabled: bool) -> Self {
        self.descriptor.enabled_by_default = enabled;
        self
    }

    pub fn with_target_modes(
        mut self,
        target_modes: impl IntoIterator<Item = RuntimeTargetMode>,
    ) -> Self {
        self.descriptor.target_modes = target_modes.into_iter().collect();
        self
    }

    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.descriptor.capabilities.push(capability.into());
        self
    }

    pub fn with_provided_interface(mut self, interface: PluginInterfaceManifest) -> Self {
        self.descriptor.provided_interfaces.push(interface);
        self
    }

    pub fn with_provided_interface_id(mut self, interface_id: impl Into<String>) -> Self {
        self.descriptor
            .provided_interfaces
            .push(PluginInterfaceManifest::new(interface_id));
        self
    }

    pub fn with_system_sets<I, S>(mut self, system_sets: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.descriptor.system_sets = system_sets.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_system_anchors<I, S>(mut self, system_anchors: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.descriptor.system_anchors = system_anchors.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_maturity(mut self, maturity: PluginMaturity) -> Self {
        self.descriptor.maturity = maturity;
        self
    }

    pub fn with_capability_status(mut self, status: CapabilityStatusManifest) -> Self {
        self.descriptor.capability_statuses.push(status);
        self
    }

    pub fn with_optional_feature(mut self, feature: PluginFeatureBundleManifest) -> Self {
        self.descriptor.optional_features.push(feature);
        self
    }

    pub fn with_default_packaging(
        mut self,
        packaging: impl IntoIterator<Item = ExportPackagingStrategy>,
    ) -> Self {
        self.descriptor.default_packaging = packaging.into_iter().collect();
        self
    }

    pub(crate) fn package_id(&self) -> &str {
        self.descriptor.package_id()
    }

    pub fn build(self) -> RuntimePluginDescriptor {
        self.descriptor
    }
}
