use serde::{Deserialize, Serialize};

use crate::core::framework::script::{ScriptHostParameterDescriptor, ScriptHostValueKind};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginInterfaceManifest {
    pub id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub methods: Vec<PluginInterfaceMethodManifest>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginInterfaceMethodManifest {
    pub name: String,
    pub method_slot: u32,
    #[serde(default = "default_bridge_method_return_value_kind")]
    pub return_value_kind: ScriptHostValueKind,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<ScriptHostParameterDescriptor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_capabilities: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
}

impl PluginInterfaceManifest {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            methods: Vec::new(),
        }
    }

    pub fn with_method(mut self, method: PluginInterfaceMethodManifest) -> Self {
        self.methods.push(method);
        self
    }

    pub fn method(&self, name: &str) -> Option<&PluginInterfaceMethodManifest> {
        self.methods.iter().find(|method| method.name == name)
    }
}

impl PluginInterfaceMethodManifest {
    pub fn new(name: impl Into<String>, method_slot: u32) -> Self {
        Self {
            name: name.into(),
            method_slot,
            return_value_kind: ScriptHostValueKind::Null,
            parameters: Vec::new(),
            required_capabilities: Vec::new(),
            documentation: None,
        }
    }

    pub fn with_return_value_kind(mut self, return_value_kind: ScriptHostValueKind) -> Self {
        self.return_value_kind = return_value_kind;
        self
    }

    pub fn with_parameter(mut self, parameter: ScriptHostParameterDescriptor) -> Self {
        self.parameters.push(parameter);
        self
    }

    pub fn with_required_capability(mut self, capability: impl Into<String>) -> Self {
        let capability = capability.into();
        if let Err(index) = self
            .required_capabilities
            .binary_search_by(|candidate| candidate.as_str().cmp(capability.as_str()))
        {
            self.required_capabilities.insert(index, capability);
        }
        self
    }

    pub fn with_documentation(mut self, documentation: impl Into<String>) -> Self {
        self.documentation = Some(documentation.into());
        self
    }
}

fn default_bridge_method_return_value_kind() -> ScriptHostValueKind {
    ScriptHostValueKind::Null
}
