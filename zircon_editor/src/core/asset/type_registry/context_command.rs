use serde::{Deserialize, Serialize};

use crate::core::editor_operation::EditorOperationPath;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetContextCommandAccess {
    #[default]
    ReadOnly,
    Mutation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetContextCommandDescriptor {
    id: String,
    display_name: String,
    operation: EditorOperationPath,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    icon_name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    required_capabilities: Vec<String>,
    #[serde(default)]
    access: AssetContextCommandAccess,
}

impl AssetContextCommandDescriptor {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        operation: EditorOperationPath,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            operation,
            icon_name: None,
            required_capabilities: Vec::new(),
            access: AssetContextCommandAccess::ReadOnly,
        }
    }

    pub fn with_icon_name(mut self, icon_name: impl Into<String>) -> Self {
        self.icon_name = Some(icon_name.into());
        self
    }

    pub fn with_required_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.required_capabilities
            .extend(capabilities.into_iter().map(Into::into));
        self.required_capabilities.sort();
        self.required_capabilities.dedup();
        self
    }

    pub fn with_mutation_access(mut self) -> Self {
        self.access = AssetContextCommandAccess::Mutation;
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn operation(&self) -> &EditorOperationPath {
        &self.operation
    }

    pub fn icon_name(&self) -> Option<&str> {
        self.icon_name.as_deref()
    }

    pub fn required_capabilities(&self) -> &[String] {
        &self.required_capabilities
    }

    pub fn access(&self) -> AssetContextCommandAccess {
        self.access
    }
}
