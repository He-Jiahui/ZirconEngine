use super::{operation_invocation, registered_command_path};
use crate::core::commands::{EditorCommandDispatchError, EditorCommandRegistry};
use crate::core::editor_operation::EditorOperationPathError;

#[test]
fn operation_binding_preserves_invalid_operation_path() {
    let error = operation_invocation("not a valid operation", &[])
        .expect_err("operation id with whitespace must be rejected");

    assert_eq!(
        error,
        EditorOperationPathError::InvalidOperationPath("not a valid operation".to_string())
    );
}

#[test]
fn editor_command_binding_preserves_unknown_command() {
    let error = registered_command_path(&EditorCommandRegistry::default(), "scene.node.missing")
        .expect_err("unregistered editor command must be rejected");

    assert_eq!(
        error,
        EditorCommandDispatchError::UnknownCommand("scene.node.missing".to_string())
    );
}
