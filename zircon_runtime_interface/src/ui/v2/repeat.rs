use serde::{Deserialize, Serialize};
use thiserror::Error;
use toml::Value;

pub const UI_V2_REPEAT_ATTRIBUTE: &str = "repeat";
pub const UI_V2_REPEAT_FIELD_KIND: &str = "kind";
pub const UI_V2_REPEAT_FIELD_PROTOTYPE: &str = "prototype";
pub const UI_V2_REPEAT_FIELD_VIRTUAL_CONTROL_PREFIX: &str = "virtual_control_prefix";
pub const UI_V2_REPEAT_FIELD_AUTHORED_COUNT: &str = "authored_count";
pub const UI_V2_REPEAT_FIELD_NODE_PATH_NAMESPACE: &str = "node_path_namespace";
pub const UI_V2_REPEAT_KIND_VIRTUAL_ROWS: &str = "virtual_rows";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiV2Repeat {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub prototype: String,
    #[serde(default)]
    pub virtual_control_prefix: String,
    #[serde(default)]
    pub authored_count: usize,
    #[serde(default)]
    pub node_path_namespace: String,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiV2RepeatValidationError {
    #[error("node {node_id} repeat.kind {kind} is unsupported; expected {expected}")]
    UnsupportedKind {
        node_id: String,
        kind: String,
        expected: &'static str,
    },
    #[error("node {node_id} repeat.prototype must not be empty")]
    EmptyPrototype { node_id: String },
    #[error("node {node_id} repeat.virtual_control_prefix must not be empty")]
    EmptyVirtualControlPrefix { node_id: String },
    #[error("node {node_id} repeat.authored_count must be greater than 0")]
    ZeroAuthoredCount { node_id: String },
}

impl UiV2Repeat {
    pub fn validate(&self, node_id: &str) -> Result<(), UiV2RepeatValidationError> {
        if self.kind != UI_V2_REPEAT_KIND_VIRTUAL_ROWS {
            return Err(UiV2RepeatValidationError::UnsupportedKind {
                node_id: node_id.to_string(),
                kind: self.kind.clone(),
                expected: UI_V2_REPEAT_KIND_VIRTUAL_ROWS,
            });
        }
        if self.prototype.trim().is_empty() {
            return Err(UiV2RepeatValidationError::EmptyPrototype {
                node_id: node_id.to_string(),
            });
        }
        if self.virtual_control_prefix.trim().is_empty() {
            return Err(UiV2RepeatValidationError::EmptyVirtualControlPrefix {
                node_id: node_id.to_string(),
            });
        }
        if self.authored_count == 0 {
            return Err(UiV2RepeatValidationError::ZeroAuthoredCount {
                node_id: node_id.to_string(),
            });
        }
        Ok(())
    }

    pub fn metadata_value(&self) -> Value {
        let mut table = toml::map::Map::new();
        table.insert(
            UI_V2_REPEAT_FIELD_KIND.to_string(),
            Value::String(self.kind.clone()),
        );
        table.insert(
            UI_V2_REPEAT_FIELD_PROTOTYPE.to_string(),
            Value::String(self.prototype.clone()),
        );
        table.insert(
            UI_V2_REPEAT_FIELD_VIRTUAL_CONTROL_PREFIX.to_string(),
            Value::String(self.virtual_control_prefix.clone()),
        );
        table.insert(
            UI_V2_REPEAT_FIELD_AUTHORED_COUNT.to_string(),
            Value::Integer(self.authored_count.min(i64::MAX as usize) as i64),
        );
        table.insert(
            UI_V2_REPEAT_FIELD_NODE_PATH_NAMESPACE.to_string(),
            Value::String(self.node_path_namespace.clone()),
        );
        Value::Table(table)
    }
}
