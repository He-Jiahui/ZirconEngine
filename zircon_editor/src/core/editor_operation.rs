use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Stable identifier shared by editor commands and operation-control DTOs.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EditorOperationPath(String);

impl EditorOperationPath {
    pub fn parse(value: impl Into<String>) -> Result<Self, EditorOperationPathError> {
        let value = value.into();
        let segments = value.split('.').collect::<Vec<_>>();
        let valid = segments
            .iter()
            .all(|segment| !segment.is_empty() && segment.chars().all(operation_path_char));
        if !valid || segments.len() < MIN_OPERATION_PATH_SEGMENTS {
            return Err(EditorOperationPathError::InvalidOperationPath(value));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<str> for EditorOperationPath {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for EditorOperationPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

const MIN_OPERATION_PATH_SEGMENTS: usize = 3;

fn operation_path_char(value: char) -> bool {
    value.is_ascii_lowercase() || value.is_ascii_digit() || value == '_'
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorOperationPathError {
    InvalidOperationPath(String),
}

impl fmt::Display for EditorOperationPathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOperationPath(path) => {
                write!(formatter, "editor operation path `{path}` is invalid")
            }
        }
    }
}

impl std::error::Error for EditorOperationPathError {}

/// Metadata retained until Editor 03 M3.2 installs the edit-command factory.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UndoableEditorOperation {
    display_name: String,
}

impl UndoableEditorOperation {
    pub fn new(display_name: impl Into<String>) -> Self {
        Self {
            display_name: display_name.into(),
        }
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorOperationInvocation {
    pub operation_id: EditorOperationPath,
    #[serde(default)]
    pub arguments: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_group: Option<String>,
}

impl EditorOperationInvocation {
    pub fn new(operation_id: EditorOperationPath) -> Self {
        Self {
            operation_id,
            arguments: Value::Null,
            operation_group: None,
        }
    }

    pub fn parse(operation_id: impl Into<String>) -> Result<Self, EditorOperationPathError> {
        Ok(Self::new(EditorOperationPath::parse(operation_id)?))
    }

    pub fn with_arguments(mut self, arguments: Value) -> Self {
        self.arguments = arguments;
        self
    }

    pub fn with_operation_group(mut self, group: impl Into<String>) -> Self {
        self.operation_group = Some(group.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorOperationSource {
    Menu,
    UiBinding,
    Remote,
    Cli,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EditorOperationControlRequest {
    InvokeOperation(EditorOperationInvocation),
    ListOperations,
    QueryOperationHistory,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EditorOperationControlResponse {
    pub operation_id: Option<String>,
    pub value: Option<Value>,
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_kind: Option<EditorOperationControlErrorKind>,
}

impl EditorOperationControlResponse {
    pub fn success(operation_id: impl Into<String>, value: Option<Value>) -> Self {
        Self {
            operation_id: Some(operation_id.into()),
            value,
            error: None,
            error_kind: None,
        }
    }

    pub fn failure(operation_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            operation_id: Some(operation_id.into()),
            value: None,
            error: Some(error.into()),
            error_kind: None,
        }
    }

    pub fn typed_failure(
        operation_id: impl Into<String>,
        error_kind: EditorOperationControlErrorKind,
        error: impl Into<String>,
    ) -> Self {
        Self {
            operation_id: Some(operation_id.into()),
            value: None,
            error: Some(error.into()),
            error_kind: Some(error_kind),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorOperationControlErrorKind {
    OperationHistoryPendingFactory,
}
