use crate::core::CoreError;

use super::super::error::{RuntimeDynamicSessionError, RuntimeProjectError};

#[test]
fn runtime_session_error_preserves_step_and_typed_source() {
    assert_eq!(
        RuntimeDynamicSessionError::ProjectStep {
            step: "load default level",
            source: RuntimeProjectError::EmptyProjectRoot,
        }
        .to_string(),
        "load default level: runtime project root cannot be empty"
    );
    assert_eq!(
        RuntimeDynamicSessionError::CoreStep {
            step: "activate runtime module",
            source: CoreError::MissingModule("script".to_string()),
        }
        .to_string(),
        "activate runtime module: module not found: script"
    );
}
