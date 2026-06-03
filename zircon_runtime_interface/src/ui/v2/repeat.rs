use serde::{Deserialize, Serialize};
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

impl UiV2Repeat {
    pub fn validate(&self, node_id: &str) -> Result<(), String> {
        if self.kind != UI_V2_REPEAT_KIND_VIRTUAL_ROWS {
            return Err(format!(
                "node {node_id} repeat.kind {} is unsupported; expected {UI_V2_REPEAT_KIND_VIRTUAL_ROWS}",
                self.kind
            ));
        }
        if self.prototype.trim().is_empty() {
            return Err(format!("node {node_id} repeat.prototype must not be empty"));
        }
        if self.virtual_control_prefix.trim().is_empty() {
            return Err(format!(
                "node {node_id} repeat.virtual_control_prefix must not be empty"
            ));
        }
        if self.authored_count == 0 {
            return Err(format!(
                "node {node_id} repeat.authored_count must be greater than 0"
            ));
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
