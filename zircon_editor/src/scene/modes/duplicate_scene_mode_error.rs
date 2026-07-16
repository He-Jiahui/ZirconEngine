use std::fmt;

use crate::core::editor_message::SceneModeId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DuplicateSceneModeError {
    mode_id: SceneModeId,
}

impl DuplicateSceneModeError {
    pub(super) fn new(mode_id: SceneModeId) -> Self {
        Self { mode_id }
    }

    pub fn mode_id(&self) -> &SceneModeId {
        &self.mode_id
    }
}

impl fmt::Display for DuplicateSceneModeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "scene mode `{}` is already active",
            self.mode_id.as_str()
        )
    }
}

impl std::error::Error for DuplicateSceneModeError {}
