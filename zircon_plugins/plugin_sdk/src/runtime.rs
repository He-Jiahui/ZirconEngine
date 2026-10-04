//! 将 SDK 包声明委托给运行时 descriptor builder，并从同一配置投影包清单。
use zircon_runtime::core::framework::project::ExportPackagingStrategy;
use zircon_runtime::core::{InitLevel, ModuleDependencySpec, ModuleDescriptor};
use zircon_runtime::plugin::{
    CapabilityStatusManifest, PluginFeatureBundleManifest, PluginInterfaceManifest, PluginMaturity,
    PluginPackageManifest, PluginPackageRole, RuntimePluginDescriptor,
    RuntimePluginDescriptorBuilder,
};
use zircon_runtime::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};

#[derive(Clone, Debug)]
/// 运行时插件 descriptor 的 fluent 声明包装，同时保留包清单投影入口。
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

    pub fn with_provided_interface(mut self, interface: PluginInterfaceManifest) -> Self {
        self.builder = self.builder.with_provided_interface(interface);
        self
    }

    pub fn with_provided_interface_id(mut self, interface_id: impl Into<String>) -> Self {
        self.builder = self.builder.with_provided_interface_id(interface_id);
        self
    }

    pub fn with_default_packaging(
        mut self,
        packaging: impl IntoIterator<Item = ExportPackagingStrategy>,
    ) -> Self {
        self.builder = self.builder.with_default_packaging(packaging);
        self
    }

    pub fn with_package_role(mut self, package_role: PluginPackageRole) -> Self {
        self.builder = self.builder.with_package_role(package_role);
        self
    }

    /// 克隆 builder 后生成 descriptor，因此不会消耗当前声明。
    pub fn descriptor(&self) -> RuntimePluginDescriptor {
        self.builder.clone().build()
    }

    /// 从与 descriptor 相同的 builder 状态生成包清单投影。
    pub fn package_manifest(&self) -> PluginPackageManifest {
        self.descriptor().package_manifest()
    }

    /// 消耗声明并直接构建最终 descriptor。
    pub fn into_descriptor(self) -> RuntimePluginDescriptor {
        self.builder.build()
    }
}

#[cfg(test)]
#[path = "tests/runtime.rs"]
mod tests;
