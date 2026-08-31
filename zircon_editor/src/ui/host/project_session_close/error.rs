use thiserror::Error;

use crate::core::recovery::ProjectSessionEffect;

use super::{ProjectCloseOperation, ProjectCloseReceipt};

#[derive(Debug, Error)]
#[error("project close failed at {effect:?}: {message}")]
pub(crate) struct ProjectCloseError {
    operation: ProjectCloseOperation,
    effect: ProjectSessionEffect,
    receipt: Option<ProjectCloseReceipt>,
    message: String,
}

impl ProjectCloseError {
    pub(crate) fn new(
        operation: ProjectCloseOperation,
        effect: ProjectSessionEffect,
        receipt: Option<ProjectCloseReceipt>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            operation,
            effect,
            receipt,
            message: message.into(),
        }
    }

    pub(crate) fn operation(&self) -> &ProjectCloseOperation {
        &self.operation
    }

    pub(crate) const fn effect(&self) -> ProjectSessionEffect {
        self.effect
    }

    pub(crate) fn receipt(&self) -> Option<&ProjectCloseReceipt> {
        self.receipt.as_ref()
    }
}
