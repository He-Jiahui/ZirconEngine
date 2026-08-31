use serde::{Deserialize, Serialize};

use super::{
    ReflectError, ReflectScriptVisibility, ReflectTypeInfo, ReflectTypePath, ReflectTypeRole,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReflectSerializationStrategy {
    None,
    Value,
    Json,
    ResourceHandle,
    EntityReference,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReflectTypeRegistration {
    pub type_path: ReflectTypePath,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
    pub type_info: ReflectTypeInfo,
    pub serialization: ReflectSerializationStrategy,
    pub role: ReflectTypeRole,
    pub serializable: bool,
    pub editor_visible: bool,
    pub remote_visible: bool,
    #[serde(default)]
    pub script_visibility: ReflectScriptVisibility,
}

impl ReflectTypeRegistration {
    pub fn new(
        type_path: ReflectTypePath,
        display_name: impl Into<String>,
        type_info: ReflectTypeInfo,
        serialization: ReflectSerializationStrategy,
    ) -> Self {
        let serializable = !matches!(serialization, ReflectSerializationStrategy::None);
        Self {
            type_path,
            display_name: display_name.into(),
            documentation: None,
            type_info,
            serialization,
            role: ReflectTypeRole::Value,
            serializable,
            editor_visible: true,
            remote_visible: false,
            script_visibility: ReflectScriptVisibility::Private,
        }
    }

    pub fn as_component(mut self) -> Self {
        self.role = ReflectTypeRole::Component;
        self
    }

    pub fn with_documentation(mut self, documentation: impl Into<String>) -> Self {
        self.documentation = Some(documentation.into());
        self
    }

    pub fn as_resource(mut self) -> Self {
        self.role = ReflectTypeRole::Resource;
        self
    }

    pub fn is_component(&self) -> bool {
        self.role == ReflectTypeRole::Component
    }

    pub fn is_resource(&self) -> bool {
        self.role == ReflectTypeRole::Resource
    }

    pub fn is_plugin_owned(&self) -> bool {
        self.type_path.plugin_id().is_some()
    }

    pub fn with_serializable(mut self, serializable: bool) -> Self {
        self.serializable = serializable;
        self
    }

    pub fn with_editor_visible(mut self, editor_visible: bool) -> Self {
        self.editor_visible = editor_visible;
        self
    }

    pub fn with_remote_visible(mut self, remote_visible: bool) -> Self {
        self.remote_visible = remote_visible;
        self
    }

    pub fn with_script_visibility(mut self, script_visibility: ReflectScriptVisibility) -> Self {
        self.script_visibility = script_visibility;
        self
    }

    pub fn with_plugin_id(mut self, plugin_id: impl Into<String>) -> Result<Self, ReflectError> {
        self.type_path = self.type_path.with_plugin_id(plugin_id)?;
        Ok(self)
    }
}
