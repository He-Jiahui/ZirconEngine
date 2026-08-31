use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandJournalPayload {
    command_type: String,
    schema_version: u16,
    payload: serde_json::Value,
}

impl CommandJournalPayload {
    pub fn new(
        command_type: impl Into<String>,
        schema_version: u16,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            command_type: command_type.into(),
            schema_version,
            payload,
        }
    }

    pub fn command_type(&self) -> &str {
        &self.command_type
    }

    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    pub fn payload(&self) -> &serde_json::Value {
        &self.payload
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandJournalUnavailable {
    label: String,
}

impl CommandJournalUnavailable {
    pub(crate) fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}
