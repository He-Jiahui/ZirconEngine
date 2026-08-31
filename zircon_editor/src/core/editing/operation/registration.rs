use std::fmt;
use std::sync::Arc;

use crate::core::editor_operation::{EditorOperationInvocation, EditorOperationPath};

use super::{
    DeferredOperationInvocation, EditOperationTarget, OperationCommand, OperationCommandFactory,
    OperationCommandFactoryError, PendingEditRetention,
};

#[derive(Clone)]
pub struct OperationCommandFactoryRegistration {
    operation: EditorOperationPath,
    undo_display_name: String,
    edit_target: EditOperationTarget,
    pending_edit_retention: PendingEditRetention,
    factory: Arc<dyn OperationCommandFactory>,
}

impl OperationCommandFactoryRegistration {
    pub fn new(
        operation: EditorOperationPath,
        undo_display_name: impl Into<String>,
        edit_target: EditOperationTarget,
        factory: Arc<dyn OperationCommandFactory>,
    ) -> Self {
        Self {
            operation,
            undo_display_name: undo_display_name.into(),
            edit_target,
            pending_edit_retention: PendingEditRetention::Lossless,
            factory,
        }
    }

    pub fn operation(&self) -> &EditorOperationPath {
        &self.operation
    }

    pub fn undo_display_name(&self) -> &str {
        &self.undo_display_name
    }

    pub const fn edit_target(&self) -> EditOperationTarget {
        self.edit_target
    }

    pub fn with_pending_edit_retention(mut self, retention: PendingEditRetention) -> Self {
        self.pending_edit_retention = retention;
        self
    }

    pub fn pending_edit_retention(&self) -> &PendingEditRetention {
        &self.pending_edit_retention
    }

    pub fn defer(
        &self,
        invocation: EditorOperationInvocation,
    ) -> Result<DeferredOperationInvocation, OperationCommandFactoryError> {
        self.ensure_matches(&invocation)?;
        Ok(DeferredOperationInvocation::from_registration(
            invocation,
            self.edit_target,
            self.pending_edit_retention.clone(),
        ))
    }

    pub fn create(
        &self,
        invocation: &EditorOperationInvocation,
    ) -> Result<OperationCommand, OperationCommandFactoryError> {
        self.ensure_matches(invocation)?;
        self.factory.create(invocation)
    }

    fn ensure_matches(
        &self,
        invocation: &EditorOperationInvocation,
    ) -> Result<(), OperationCommandFactoryError> {
        if self.operation != invocation.operation_id {
            return Err(OperationCommandFactoryError::OperationMismatch {
                descriptor_operation: invocation.operation_id.clone(),
                factory_operation: self.operation.clone(),
            });
        }
        Ok(())
    }
}

impl fmt::Debug for OperationCommandFactoryRegistration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OperationCommandFactoryRegistration")
            .field("operation", &self.operation)
            .field("undo_display_name", &self.undo_display_name)
            .field("edit_target", &self.edit_target)
            .field("pending_edit_retention", &self.pending_edit_retention)
            .finish_non_exhaustive()
    }
}

impl PartialEq for OperationCommandFactoryRegistration {
    fn eq(&self, other: &Self) -> bool {
        self.operation == other.operation
            && self.undo_display_name == other.undo_display_name
            && self.edit_target == other.edit_target
            && self.pending_edit_retention == other.pending_edit_retention
            && Arc::ptr_eq(&self.factory, &other.factory)
    }
}
