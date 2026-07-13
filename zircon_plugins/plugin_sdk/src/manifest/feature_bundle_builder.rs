use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ExportPackagingStrategy;
use zircon_runtime::plugin::{
    PluginFeatureBundleManifest, PluginFeatureDependency, PluginModuleManifest,
};

use super::{default_export_packaging, PluginModuleBuilder};

#[derive(Clone, Debug)]
pub struct PluginFeatureBundleBuilder {
    feature: PluginFeatureBundleManifest,
}

impl PluginFeatureBundleBuilder {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        owner_plugin_id: impl Into<String>,
    ) -> Self {
        Self {
            feature: PluginFeatureBundleManifest::new(id, display_name, owner_plugin_id)
                .with_default_packaging(default_export_packaging()),
        }
    }

    pub fn with_dependency(mut self, dependency: PluginFeatureDependency) -> Self {
        self.feature = self.feature.with_dependency(dependency);
        self
    }

    pub fn with_primary_dependency(
        self,
        plugin_id: impl Into<String>,
        capability: impl Into<String>,
    ) -> Self {
        self.with_dependency(PluginFeatureDependency::primary(plugin_id, capability))
    }

    pub fn with_required_dependency(
        self,
        plugin_id: impl Into<String>,
        capability: impl Into<String>,
    ) -> Self {
        self.with_dependency(PluginFeatureDependency::required(plugin_id, capability))
    }

    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.feature = self.feature.with_capability(capability);
        self
    }

    pub fn with_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for capability in capabilities {
            self = self.with_capability(capability);
        }
        self
    }

    pub fn with_runtime_capability_module<I, S>(
        self,
        capability: impl Into<String>,
        module_name: impl Into<String>,
        crate_name: impl Into<String>,
        target_modes: I,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<RuntimeTargetMode>,
    {
        let capability = capability.into();
        let target_modes = target_modes.into_iter().map(Into::into);
        let module = PluginModuleManifest::runtime(module_name, crate_name)
            .with_target_modes(target_modes)
            .with_capabilities([capability.clone()]);
        self.with_capability(capability).with_runtime_module(module)
    }

    pub fn with_editor_capability_module(
        self,
        capability: impl Into<String>,
        module_name: impl Into<String>,
        crate_name: impl Into<String>,
    ) -> Self {
        let capability = capability.into();
        let module = PluginModuleManifest::editor(module_name, crate_name)
            .with_capabilities([capability.clone()]);
        self.with_capability(capability).with_editor_module(module)
    }

    pub fn with_runtime_module(mut self, module: PluginModuleManifest) -> Self {
        self.feature = self.feature.with_runtime_module(module);
        self
    }

    pub fn with_runtime_module_from_builder(self, module: PluginModuleBuilder) -> Self {
        self.with_runtime_module(module.build())
    }

    pub fn with_editor_module(mut self, module: PluginModuleManifest) -> Self {
        self.feature = self.feature.with_editor_module(module);
        self
    }

    pub fn with_editor_module_from_builder(self, module: PluginModuleBuilder) -> Self {
        self.with_editor_module(module.build())
    }

    pub fn with_default_packaging(
        mut self,
        packaging: impl IntoIterator<Item = ExportPackagingStrategy>,
    ) -> Self {
        self.feature = self.feature.with_default_packaging(packaging);
        self
    }

    pub fn enabled_by_default(mut self, enabled: bool) -> Self {
        self.feature = self.feature.enabled_by_default(enabled);
        self
    }

    pub fn build(self) -> PluginFeatureBundleManifest {
        self.feature
    }
}
