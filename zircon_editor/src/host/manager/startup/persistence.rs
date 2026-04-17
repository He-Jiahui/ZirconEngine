use serde_json::Value;

use crate::workbench::startup::{StoredStartupSession, STARTUP_SESSION_KEY};

use super::super::editor_error::EditorError;
use super::super::editor_manager::EditorManager;

impl EditorManager {
    pub(super) fn load_startup_session(&self) -> Result<StoredStartupSession, EditorError> {
        let Some(value) = self.config_manager()?.get_value(STARTUP_SESSION_KEY) else {
            return Ok(StoredStartupSession::default());
        };
        serde_json::from_value::<StoredStartupSession>(Value::from(value))
            .map_err(|error| EditorError::Project(error.to_string()))
    }

    pub(super) fn save_startup_session(
        &self,
        session: &StoredStartupSession,
    ) -> Result<(), EditorError> {
        self.config_manager()?
            .set_value(
                STARTUP_SESSION_KEY,
                serde_json::to_value(session)
                    .map_err(|error| EditorError::Project(error.to_string()))?,
            )
            .map_err(|error| EditorError::Project(error.to_string()))
    }
}
