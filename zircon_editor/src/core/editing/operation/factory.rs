use crate::core::editor_operation::EditorOperationInvocation;

use super::{OperationCommand, OperationCommandFactoryError};

pub trait OperationCommandFactory: Send + Sync {
    fn create(
        &self,
        invocation: &EditorOperationInvocation,
    ) -> Result<OperationCommand, OperationCommandFactoryError>;
}
