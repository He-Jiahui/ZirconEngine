use serde_json::Value;

use crate::core::project::{ProjectAuthority, StoredStartupSession};

const STARTUP_SESSION_KEY: &str = "editor.startup.session";

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;

impl EditorUiHost {
    pub(super) fn load_startup_session(&self) -> Result<StoredStartupSession, EditorError> {
        let Some(value) = self.config_manager()?.get_value(STARTUP_SESSION_KEY) else {
            return Ok(StoredStartupSession::default());
        };
        Ok(ProjectAuthority::default().decode_startup_session(Value::from(value))?)
    }

    pub(super) fn save_startup_session(
        &self,
        session: &StoredStartupSession,
    ) -> Result<(), EditorError> {
        self.config_manager()?
            .set_value(
                STARTUP_SESSION_KEY,
                ProjectAuthority::default().encode_startup_session(session)?,
            )
            .map_err(|error| EditorError::Project(error.to_string()))
    }
}
