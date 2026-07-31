use std::fmt;

use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// Stable identifier shared by editor commands and operation-control DTOs.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct EditorOperationPath(String);

impl EditorOperationPath {
    pub fn parse(value: impl Into<String>) -> Result<Self, EditorOperationPathError> {
        let value = value.into();
        if !Self::is_valid(&value) {
            return Err(EditorOperationPathError::InvalidOperationPath(value));
        }
        Ok(Self(value))
    }

    pub(crate) fn is_valid(value: &str) -> bool {
        let mut segment_count = 0;
        let valid = value.split('.').all(|segment| {
            segment_count += 1;
            !segment.is_empty() && segment.chars().all(operation_path_char)
        });
        valid && segment_count >= MIN_OPERATION_PATH_SEGMENTS
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

impl<'de> Deserialize<'de> for EditorOperationPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(de::Error::custom)
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
}

impl EditorOperationControlResponse {
    pub fn success(operation_id: impl Into<String>, value: Option<Value>) -> Self {
        Self {
            operation_id: Some(operation_id.into()),
            value,
            error: None,
        }
    }

    pub fn failure(operation_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            operation_id: Some(operation_id.into()),
            value: None,
            error: Some(error.into()),
        }
    }
}
