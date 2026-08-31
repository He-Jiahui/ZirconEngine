use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::{InitLevel, ModuleDependencySpec};
use zircon_runtime::plugin::{PluginModuleKind, PluginModuleManifest};

fn join_module_metadata(parts: &[&str]) -> String {
    let capacity = parts.iter().map(|part| part.len()).sum();
    let mut joined = String::with_capacity(capacity);
    for part in parts {
        joined.push_str(part);
    }
    joined
}

#[derive(Clone, Debug)]
pub struct PluginModuleBuilder {
    module: PluginModuleManifest,
}

impl PluginModuleBuilder {
    pub fn runtime(package_id: impl AsRef<str>, crate_name: impl Into<String>) -> Self {
        Self::new(
            join_module_metadata(&[package_id.as_ref(), ".runtime"]),
            PluginModuleKind::Runtime,
            crate_name,
        )
    }

    pub fn editor(package_id: impl AsRef<str>, crate_name: impl Into<String>) -> Self {
        Self::new(
            join_module_metadata(&[package_id.as_ref(), ".editor"]),
            PluginModuleKind::Editor,
            crate_name,
        )
        .with_target_modes([RuntimeTargetMode::EditorHost])
    }

    pub fn native(package_id: impl AsRef<str>, crate_name: impl Into<String>) -> Self {
        Self::new(
            join_module_metadata(&[package_id.as_ref(), ".native"]),
            PluginModuleKind::Native,
            crate_name,
        )
    }

    pub fn vm(package_id: impl AsRef<str>, crate_name: impl Into<String>) -> Self {
        Self::new(
            join_module_metadata(&[package_id.as_ref(), ".vm"]),
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
                description: join_module_metadata(&["Plugin module ", &name]),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_module_metadata_preserves_builtin_names_and_descriptions() {
        let modules = [
            PluginModuleBuilder::runtime("weather", "weather_runtime").build(),
            PluginModuleBuilder::editor("weather", "weather_editor").build(),
            PluginModuleBuilder::native("weather", "weather_native").build(),
            PluginModuleBuilder::vm("weather", "weather_vm").build(),
        ];
        let expected_names = [
            "weather.runtime",
            "weather.editor",
            "weather.native",
            "weather.vm",
        ];

        for (module, expected_name) in modules.iter().zip(expected_names) {
            assert_eq!(module.name, expected_name);
            assert_eq!(module.description, format!("Plugin module {expected_name}"));
        }
    }
}
