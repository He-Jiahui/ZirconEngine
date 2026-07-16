use std::fmt;
use std::sync::Arc;

use crate::core::editor_operation::{EditorOperationInvocation, EditorOperationPath};

use super::{OperationCommand, OperationCommandFactory, OperationCommandFactoryError};

#[derive(Clone)]
pub struct OperationCommandFactoryRegistration {
    operation: EditorOperationPath,
    undo_display_name: String,
    factory: Arc<dyn OperationCommandFactory>,
}

impl OperationCommandFactoryRegistration {
    pub fn new(
        operation: EditorOperationPath,
        undo_display_name: impl Into<String>,
        factory: Arc<dyn OperationCommandFactory>,
    ) -> Self {
        Self {
            operation,
            undo_display_name: undo_display_name.into(),
            factory,
        }
    }

    pub fn operation(&self) -> &EditorOperationPath {
        &self.operation
    }

    pub fn undo_display_name(&self) -> &str {
        &self.undo_display_name
    }

    pub fn create(
        &self,
        invocation: &EditorOperationInvocation,
    ) -> Result<OperationCommand, OperationCommandFactoryError> {
        if self.operation != invocation.operation_id {
            return Err(OperationCommandFactoryError::OperationMismatch {
                descriptor_operation: invocation.operation_id.clone(),
                factory_operation: self.operation.clone(),
            });
        }
        self.factory.create(invocation)
    }
}

impl fmt::Debug for OperationCommandFactoryRegistration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OperationCommandFactoryRegistration")
            .field("operation", &self.operation)
            .field("undo_display_name", &self.undo_display_name)
            .finish_non_exhaustive()
    }
}

impl PartialEq for OperationCommandFactoryRegistration {
    fn eq(&self, other: &Self) -> bool {
        self.operation == other.operation
            && self.undo_display_name == other.undo_display_name
            && Arc::ptr_eq(&self.factory, &other.factory)
    }
}
