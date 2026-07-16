use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::{InitLevel, ModuleDependencySpec};
use zircon_runtime::plugin::{PluginModuleKind, PluginModuleManifest};

#[derive(Clone, Debug)]
pub struct PluginModuleBuilder {
    module: PluginModuleManifest,
}

impl PluginModuleBuilder {
    pub fn runtime(package_id: impl AsRef<str>, crate_name: impl Into<String>) -> Self {
        Self::new(
            format!("{}.runtime", package_id.as_ref()),
            PluginModuleKind::Runtime,
            crate_name,
        )
    }

    pub fn editor(package_id: impl AsRef<str>, crate_name: impl Into<String>) -> Self {
        Self::new(
            format!("{}.editor", package_id.as_ref()),
            PluginModuleKind::Editor,
            crate_name,
        )
        .with_target_modes([RuntimeTargetMode::EditorHost])
    }

    pub fn native(package_id: impl AsRef<str>, crate_name: impl Into<String>) -> Self {
        Self::new(
            format!("{}.native", package_id.as_ref()),
            PluginModuleKind::Native,
            crate_name,
        )
    }

    pub fn vm(package_id: impl AsRef<str>, crate_name: impl Into<String>) -> Self {
        Self::new(
            format!("{}.vm", package_id.as_ref()),
            PluginModuleKind::Vm,
            crate_name,
        )
    }

    pub fn new(
        name: impl Into<String>,
        kind: PluginModuleKind,
        crate_name: impl Into<String>,
    ) -> Self {
        let name = name.into();
        Self {
            module: PluginModuleManifest {
                description: format!("Plugin module {name}"),
                name,
                kind,
                crate_name: crate_name.into(),
                init_level: InitLevel::Post,
                module_dependencies: Vec::new(),
                target_modes: Vec::new(),
                capabilities: Vec::new(),
                system_sets: Vec::new(),
                system_anchors: Vec::new(),
                event_consumers: Vec::new(),
            },
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.module.description = description.into();
        self
    }

    pub fn with_init_level(mut self, init_level: InitLevel) -> Self {
        self.module.init_level = init_level;
        self
    }

    pub fn with_module_dependency(mut self, dependency: ModuleDependencySpec) -> Self {
        self.module.module_dependencies.push(dependency);
        self
    }

    pub fn with_module_dependencies(
        mut self,
        dependencies: impl IntoIterator<Item = ModuleDependencySpec>,
    ) -> Self {
        self.module.module_dependencies = dependencies.into_iter().collect();
        self
    }

    pub fn with_target_modes(
        mut self,
        target_modes: impl IntoIterator<Item = RuntimeTargetMode>,
    ) -> Self {
        self.module.target_modes = target_modes.into_iter().collect();
        self
    }

    pub fn with_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.module.capabilities = capabilities.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_system_sets<I, S>(mut self, system_sets: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.module.system_sets = system_sets.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_system_anchors<I, S>(mut self, system_anchors: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.module.system_anchors = system_anchors.into_iter().map(Into::into).collect();
        self
    }

    pub fn build(self) -> PluginModuleManifest {
        self.module
    }
}
