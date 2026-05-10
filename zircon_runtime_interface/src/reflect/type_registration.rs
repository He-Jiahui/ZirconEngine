use serde::{Deserialize, Serialize};

use super::{ReflectTypeInfo, ReflectTypePath};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReflectSerializationStrategy {
    None,
    Value,
    Json,
    ResourceHandle,
    EntityReference,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReflectTypeRegistration {
    pub type_path: ReflectTypePath,
    pub display_name: String,
    pub type_info: ReflectTypeInfo,
    pub serialization: ReflectSerializationStrategy,
    pub is_component: bool,
    pub is_resource: bool,
    pub plugin_owned: bool,
    pub serializable: bool,
    pub editor_visible: bool,
    pub remote_visible: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_id: Option<String>,
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
            type_info,
            serialization,
            is_component: false,
            is_resource: false,
            plugin_owned: false,
            serializable,
            editor_visible: true,
            remote_visible: false,
            plugin_id: None,
        }
    }

    pub fn as_component(mut self) -> Self {
        self.is_component = true;
        self
    }

    pub fn as_resource(mut self) -> Self {
        self.is_resource = true;
        self
    }

    pub fn with_plugin_owned(mut self, plugin_owned: bool) -> Self {
        self.plugin_owned = plugin_owned;
        self
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

    pub fn with_plugin_id(mut self, plugin_id: impl Into<String>) -> Self {
        let plugin_id = plugin_id.into();
        self.type_path.plugin_id = Some(plugin_id.clone());
        self.plugin_id = Some(plugin_id);
        self
    }
}
