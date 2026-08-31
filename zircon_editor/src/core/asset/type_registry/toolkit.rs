use serde::{Deserialize, Serialize};

use crate::core::editor_operation::EditorOperationPath;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetToolkitDescriptor {
    view_id: String,
    open_operation: EditorOperationPath,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    required_capabilities: Vec<String>,
}

impl AssetToolkitDescriptor {
    pub fn new(view_id: impl Into<String>, open_operation: EditorOperationPath) -> Self {
        Self {
            view_id: view_id.into(),
            open_operation,
            required_capabilities: Vec::new(),
        }
    }

    pub fn with_required_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let capabilities = capabilities.into_iter();
        let (lower_bound, _) = capabilities.size_hint();
        self.required_capabilities.reserve(lower_bound);
        self.required_capabilities
            .extend(capabilities.map(Into::into));
        self.required_capabilities.sort_unstable();
        self.required_capabilities.dedup();
        self
    }

    pub fn view_id(&self) -> &str {
        &self.view_id
    }

    pub fn open_operation(&self) -> &EditorOperationPath {
        &self.open_operation
    }

    pub fn required_capabilities(&self) -> &[String] {
        &self.required_capabilities
    }
}

#[cfg(test)]
#[path = "toolkit/optimization_tests.rs"]
mod optimization_tests;
