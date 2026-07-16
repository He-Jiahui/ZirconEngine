use thiserror::Error;

use crate::core::editor_operation::EditorOperationPath;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum OperationCommandFactoryError {
    #[error(
        "editor operation descriptor {descriptor_operation} does not match factory {factory_operation}"
    )]
    OperationMismatch {
        descriptor_operation: EditorOperationPath,
        factory_operation: EditorOperationPath,
    },
    #[error(
        "editor operation {operation} uses an event route and cannot install a command factory"
    )]
    DescriptorIsEvent { operation: EditorOperationPath },
    #[error("editor operation {operation} already has a command factory")]
    DuplicateFactory { operation: EditorOperationPath },
    #[error("editor operation {operation} has no installed command factory")]
    MissingFactory { operation: EditorOperationPath },
    #[error("editor operation factory {operation} has no matching command descriptor")]
    OrphanFactory { operation: EditorOperationPath },
    #[error("editor operation {operation} has invalid arguments: {reason}")]
    InvalidArguments {
        operation: EditorOperationPath,
        reason: String,
    },
    #[error("editor operation {operation} command factory failed: {reason}")]
    Factory {
        operation: EditorOperationPath,
        reason: String,
    },
}
