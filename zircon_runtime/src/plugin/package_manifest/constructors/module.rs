use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::{InitLevel, ModuleDependencySpec, ModuleDescriptor};

use super::super::{PluginModuleKind, PluginModuleManifest};

impl PluginModuleManifest {
    pub fn runtime(name: impl Into<String>, crate_name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            description: default_module_description(PluginModuleKind::Runtime, &name),
            name,
            kind: PluginModuleKind::Runtime,
            crate_name: crate_name.into(),
            init_level: InitLevel::Post,
            module_dependencies: Vec::new(),
            target_modes: Vec::new(),
            capabilities: Vec::new(),
            system_sets: Vec::new(),
            system_anchors: Vec::new(),
            event_consumers: Vec::new(),
        }
    }

    pub fn editor(name: impl Into<String>, crate_name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            description: default_module_description(PluginModuleKind::Editor, &name),
            name,
            kind: PluginModuleKind::Editor,
            crate_name: crate_name.into(),
            init_level: InitLevel::Post,
            module_dependencies: Vec::new(),
            target_modes: vec![RuntimeTargetMode::EditorHost],
            capabilities: Vec::new(),
            system_sets: Vec::new(),
            system_anchors: Vec::new(),
            event_consumers: Vec::new(),
        }
    }

    pub fn native(name: impl Into<String>, crate_name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            description: default_module_description(PluginModuleKind::Native, &name),
            name,
            kind: PluginModuleKind::Native,
            crate_name: crate_name.into(),
            init_level: InitLevel::Post,
            module_dependencies: Vec::new(),
            target_modes: Vec::new(),
            capabilities: Vec::new(),
            system_sets: Vec::new(),
            system_anchors: Vec::new(),
            event_consumers: Vec::new(),
        }
    }

    pub fn vm(name: impl Into<String>, crate_name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            description: default_module_description(PluginModuleKind::Vm, &name),
            name,
            kind: PluginModuleKind::Vm,
            crate_name: crate_name.into(),
            init_level: InitLevel::Post,
            module_dependencies: Vec::new(),
            target_modes: Vec::new(),
            capabilities: Vec::new(),
            system_sets: Vec::new(),
            system_anchors: Vec::new(),
            event_consumers: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_init_level(mut self, init_level: InitLevel) -> Self {
        self.init_level = init_level;
        self
    }

    pub fn with_module_dependency(mut self, dependency: ModuleDependencySpec) -> Self {
        self.module_dependencies.push(dependency);
        self
    }

    pub fn with_module_dependencies(
        mut self,
        dependencies: impl IntoIterator<Item = ModuleDependencySpec>,
    ) -> Self {
        self.module_dependencies = dependencies.into_iter().collect();
        self
    }

    pub fn module_descriptor(&self) -> ModuleDescriptor {
        let description = if self.description.is_empty() {
            default_module_description(self.kind, &self.name)
        } else {
            self.description.clone()
        };
        let mut descriptor =
            ModuleDescriptor::new(self.name.clone(), description).with_init_level(self.init_level);
        for dependency in self.module_dependencies.iter().cloned() {
            descriptor = descriptor.with_module_dependency(dependency);
        }
        descriptor
    }

    pub fn with_target_modes(
        mut self,
        target_modes: impl IntoIterator<Item = RuntimeTargetMode>,
    ) -> Self {
        self.target_modes = target_modes.into_iter().collect();
        self
    }

    pub fn with_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.capabilities = capabilities.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_system_sets<I, S>(mut self, system_sets: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.system_sets = system_sets.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_system_anchors<I, S>(mut self, system_anchors: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.system_anchors = system_anchors.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_event_consumer(mut self, consumer: super::PluginEventConsumerManifest) -> Self {
        self.event_consumers.push(consumer);
        self
    }

    pub fn with_event_consumers(
        mut self,
        consumers: impl IntoIterator<Item = super::PluginEventConsumerManifest>,
    ) -> Self {
        self.event_consumers = consumers.into_iter().collect();
        self
    }
}

fn default_module_description(kind: PluginModuleKind, name: &str) -> String {
    let label = match kind {
        PluginModuleKind::Runtime => "Runtime",
        PluginModuleKind::Editor => "Editor",
        PluginModuleKind::Native => "Native",
        PluginModuleKind::Vm => "VM",
    };
    format!("{label} plugin module {name}")
}
