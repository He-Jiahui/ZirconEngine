use serde::{Deserialize, Serialize};

const AUTOSAVE_ENGINE_SCHEMA_ID: &str = "zircon_editor.autosave.snapshot";
const AUTOSAVE_ENGINE_SCHEMA_VERSION: u32 = 1;

/// Identifies the recovery payload contract independently of a document file extension.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutosaveEngineSchema {
    engine_id: String,
    schema_version: u32,
}

impl AutosaveEngineSchema {
    pub fn current() -> Self {
        Self {
            engine_id: AUTOSAVE_ENGINE_SCHEMA_ID.to_string(),
            schema_version: AUTOSAVE_ENGINE_SCHEMA_VERSION,
        }
    }

    pub fn engine_id(&self) -> &str {
        &self.engine_id
    }

    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub(crate) fn is_current(&self) -> bool {
        self.engine_id == AUTOSAVE_ENGINE_SCHEMA_ID
            && self.schema_version == AUTOSAVE_ENGINE_SCHEMA_VERSION
    }
}
