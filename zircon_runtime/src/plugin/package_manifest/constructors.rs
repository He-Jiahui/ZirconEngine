use crate::{
    ComponentTypeDescriptor, ExportPackagingStrategy, RuntimeTargetMode, UiComponentDescriptor,
};

use super::{PluginModuleKind, PluginModuleManifest, PluginPackageManifest};

impl PluginModuleManifest {
    pub fn runtime(name: impl Into<String>, crate_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: PluginModuleKind::Runtime,
            crate_name: crate_name.into(),
            target_modes: Vec::new(),
            capabilities: Vec::new(),
        }
    }

    pub fn editor(name: impl Into<String>, crate_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: PluginModuleKind::Editor,
            crate_name: crate_name.into(),
            target_modes: vec![RuntimeTargetMode::EditorHost],
            capabilities: Vec::new(),
        }
    }

    pub fn with_target_modes(
        mut self,
        target_modes: impl IntoIterator<Item = RuntimeTargetMode>,
    ) -> Self {
        self.target_modes = target_modes.into_iter().collect();
        self
    }

    pub fn with_capabilities(mut self, capabilities: impl IntoIterator<Item = String>) -> Self {
        self.capabilities = capabilities.into_iter().collect();
        self
    }
}

impl PluginPackageManifest {
    pub fn new(id: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            version: "0.1.0".to_string(),
            display_name: display_name.into(),
            description: String::new(),
            modules: Vec::new(),
            components: Vec::new(),
            ui_components: Vec::new(),
            default_packaging: vec![
                ExportPackagingStrategy::SourceTemplate,
                ExportPackagingStrategy::LibraryEmbed,
            ],
        }
    }

    pub fn with_runtime_crate(mut self, crate_name: impl Into<String>) -> Self {
        self.modules.push(PluginModuleManifest::runtime(
            format!("{}.runtime", self.id),
            crate_name,
        ));
        self
    }

    pub fn with_runtime_module(mut self, module: PluginModuleManifest) -> Self {
        self.modules.push(module);
        self
    }

    pub fn with_component(mut self, descriptor: ComponentTypeDescriptor) -> Self {
        self.components.push(descriptor);
        self
    }

    pub fn with_ui_component(mut self, descriptor: UiComponentDescriptor) -> Self {
        self.ui_components.push(descriptor);
        self
    }

    pub fn with_editor_crate(mut self, crate_name: impl Into<String>) -> Self {
        self.modules.push(PluginModuleManifest::editor(
            format!("{}.editor", self.id),
            crate_name,
        ));
        self
    }

    pub fn with_editor_module(mut self, module: PluginModuleManifest) -> Self {
        self.modules.push(module);
        self
    }
}
