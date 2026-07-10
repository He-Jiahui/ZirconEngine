use zircon_runtime::builtin::{RuntimePluginId, RuntimeTargetMode};
use zircon_runtime::core::{InitLevel, ModuleDependencySpec, ModuleDescriptor};
use zircon_runtime::plugin::{
    CapabilityStatusManifest, ExportPackagingStrategy, PluginFeatureBundleManifest, PluginMaturity,
    PluginPackageManifest, RuntimePluginDescriptor, RuntimePluginDescriptorBuilder,
};

#[derive(Clone, Debug)]
pub struct RuntimePluginDeclaration {
    builder: RuntimePluginDescriptorBuilder,
}

impl RuntimePluginDeclaration {
    pub fn new(
        package_id: impl Into<String>,
        display_name: impl Into<String>,
        runtime_id: RuntimePluginId,
        crate_name: impl Into<String>,
    ) -> Self {
        Self {
            builder: RuntimePluginDescriptor::builder(
                package_id,
                display_name,
                runtime_id,
                crate_name,
            ),
        }
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.builder = self.builder.with_category(category);
        self
    }

    pub fn with_enabled_by_default(mut self, enabled: bool) -> Self {
        self.builder = self.builder.with_enabled_by_default(enabled);
        self
    }

    pub fn with_required_by_default(mut self, required: bool) -> Self {
        self.builder = self.builder.with_required_by_default(required);
        self
    }

    pub fn with_target_modes(
        mut self,
        target_modes: impl IntoIterator<Item = RuntimeTargetMode>,
    ) -> Self {
        self.builder = self.builder.with_target_modes(target_modes);
        self
    }

    pub fn with_init_level(mut self, init_level: InitLevel) -> Self {
        self.builder = self.builder.with_init_level(init_level);
        self
    }

    pub fn with_module_descriptor(mut self, descriptor: ModuleDescriptor) -> Self {
        self.builder = self.builder.with_module_descriptor(descriptor);
        self
    }

    pub fn with_module_dependency(mut self, dependency: ModuleDependencySpec) -> Self {
        self.builder = self.builder.with_module_dependency(dependency);
        self
    }

    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.builder = self.builder.with_capability(capability);
        self
    }

    pub fn with_system_sets<I, S>(mut self, system_sets: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.builder = self.builder.with_system_sets(system_sets);
        self
    }

    pub fn with_system_anchors<I, S>(mut self, system_anchors: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.builder = self.builder.with_system_anchors(system_anchors);
        self
    }

    pub fn with_maturity(mut self, maturity: PluginMaturity) -> Self {
        self.builder = self.builder.with_maturity(maturity);
        self
    }

    pub fn with_capability_status(mut self, status: CapabilityStatusManifest) -> Self {
        self.builder = self.builder.with_capability_status(status);
        self
    }

    pub fn with_optional_feature(mut self, feature: PluginFeatureBundleManifest) -> Self {
        self.builder = self.builder.with_optional_feature(feature);
        self
    }

    pub fn with_default_packaging(
        mut self,
        packaging: impl IntoIterator<Item = ExportPackagingStrategy>,
    ) -> Self {
        self.builder = self.builder.with_default_packaging(packaging);
        self
    }

    pub fn descriptor(&self) -> RuntimePluginDescriptor {
        self.builder.clone().build()
    }

    pub fn package_manifest(&self) -> PluginPackageManifest {
        self.descriptor().package_manifest()
    }

    pub fn into_descriptor(self) -> RuntimePluginDescriptor {
        self.builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_declaration_projects_descriptor_and_manifest_from_one_source() {
        let declaration = RuntimePluginDeclaration::new(
            "navigation",
            "Navigation",
            RuntimePluginId::Navigation,
            "zircon_plugin_navigation_runtime",
        )
        .with_category("runtime")
        .with_maturity(PluginMaturity::Beta)
        .with_target_modes([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::ServerRuntime,
        ])
        .with_capability("runtime.plugin.navigation")
        .with_system_anchors(["navigation.runtime.tick"]);

        let descriptor = declaration.descriptor();
        let manifest = declaration.package_manifest();

        assert_eq!(descriptor.package_id(), "navigation");
        assert_eq!(
            descriptor.capabilities(),
            ["runtime.plugin.navigation".to_string()]
        );
        assert_eq!(manifest.id, descriptor.package_id());
        assert_eq!(manifest.category, descriptor.category());
        assert_eq!(manifest.maturity, descriptor.maturity());
        assert_eq!(manifest.supported_targets, descriptor.target_modes());
        assert_eq!(manifest.capabilities, descriptor.capabilities());
        assert!(manifest
            .modules
            .iter()
            .any(|module| module.name == "navigation.runtime"
                && module.crate_name == "zircon_plugin_navigation_runtime"
                && module.system_anchors == ["navigation.runtime.tick".to_string()]));
    }
}
