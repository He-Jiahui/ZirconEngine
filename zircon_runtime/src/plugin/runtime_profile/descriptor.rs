use serde::{Deserialize, Serialize};

use crate::core::framework::project::{
    ProjectPluginManifest, ProjectPluginSelection, RuntimeProfileId,
};
use crate::plugin::PluginMaturity;
use crate::{
    builtin::{BuiltinRuntimeModuleId, RuntimePluginId},
    core::framework::platform::RuntimeTargetMode,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeProfilePluginSelection {
    pub id: RuntimePluginId,
    #[serde(default)]
    pub required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeProfileDescriptor {
    pub id: RuntimeProfileId,
    pub name: String,
    pub target_mode: RuntimeTargetMode,
    pub builtin_modules: Vec<BuiltinRuntimeModuleId>,
    #[serde(default)]
    pub default_plugins: Vec<RuntimeProfilePluginSelection>,
    #[serde(default)]
    pub optional_plugins: Vec<RuntimePluginId>,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
    pub minimum_maturity: PluginMaturity,
    #[serde(default)]
    pub allow_externalized_required_plugins: bool,
}

impl RuntimeProfilePluginSelection {
    pub fn new(id: RuntimePluginId, required: bool) -> Self {
        Self { id, required }
    }
}

impl RuntimeProfileDescriptor {
    pub fn new(
        id: RuntimeProfileId,
        name: impl Into<String>,
        target_mode: RuntimeTargetMode,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            target_mode,
            builtin_modules: Vec::new(),
            default_plugins: Vec::new(),
            optional_plugins: Vec::new(),
            required_capabilities: Vec::new(),
            minimum_maturity: PluginMaturity::Experimental,
            allow_externalized_required_plugins: false,
        }
    }

    pub fn with_builtin_module(mut self, id: BuiltinRuntimeModuleId) -> Self {
        if !self.builtin_modules.contains(&id) {
            self.builtin_modules.push(id);
        }
        self
    }

    pub fn with_builtin_modules(
        mut self,
        ids: impl IntoIterator<Item = BuiltinRuntimeModuleId>,
    ) -> Self {
        for id in ids {
            if !self.builtin_modules.contains(&id) {
                self.builtin_modules.push(id);
            }
        }
        self
    }

    pub fn with_default_plugin(mut self, id: RuntimePluginId, required: bool) -> Self {
        self.default_plugins
            .push(RuntimeProfilePluginSelection::new(id, required));
        self
    }

    pub fn with_optional_plugin(mut self, id: RuntimePluginId) -> Self {
        if !self.optional_plugins.contains(&id) {
            self.optional_plugins.push(id);
        }
        self
    }

    pub fn with_required_capability(mut self, capability: impl Into<String>) -> Self {
        self.required_capabilities.push(capability.into());
        self
    }

    pub fn with_minimum_maturity(mut self, maturity: PluginMaturity) -> Self {
        self.minimum_maturity = maturity;
        self
    }

    pub fn allow_externalized_required_plugins(mut self, allow: bool) -> Self {
        self.allow_externalized_required_plugins = allow;
        self
    }

    pub fn project_manifest(&self) -> ProjectPluginManifest {
        ProjectPluginManifest {
            selections: self
                .default_plugins
                .iter()
                .map(|plugin| {
                    ProjectPluginSelection::runtime_plugin(plugin.id.clone(), true, plugin.required)
                        .with_target_modes([self.target_mode])
                })
                .collect(),
        }
    }
}
