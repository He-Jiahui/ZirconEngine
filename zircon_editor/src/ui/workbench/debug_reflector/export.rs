use std::fmt;

use zircon_runtime_interface::ui::surface::UiSurfaceDebugSnapshot;

#[derive(Debug)]
pub(crate) enum EditorUiDebugReflectorExportError {
    Serialize(serde_json::Error),
    Deserialize(serde_json::Error),
}

impl fmt::Display for EditorUiDebugReflectorExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialize(error) => write!(f, "failed to serialize UI debug snapshot: {error}"),
            Self::Deserialize(error) => {
                write!(f, "failed to parse UI debug snapshot JSON: {error}")
            }
        }
    }
}

impl std::error::Error for EditorUiDebugReflectorExportError {}

pub(crate) fn load_snapshot_json(
    text: &str,
) -> Result<UiSurfaceDebugSnapshot, EditorUiDebugReflectorExportError> {
    serde_json::from_str(text).map_err(EditorUiDebugReflectorExportError::Deserialize)
}

pub(crate) fn snapshot_to_json(
    snapshot: &UiSurfaceDebugSnapshot,
) -> Result<String, EditorUiDebugReflectorExportError> {
    serde_json::to_string_pretty(snapshot).map_err(EditorUiDebugReflectorExportError::Serialize)
}
