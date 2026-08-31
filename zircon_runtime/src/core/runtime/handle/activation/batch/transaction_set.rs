use crate::core::CoreError;

use super::super::super::super::state::{ModuleLifecycleCommand, ModuleLifecycleTransitionToken};
use super::super::super::CoreHandle;

pub(super) struct LifecycleTransactionSet<'a> {
    handle: &'a CoreHandle,
    tokens: Vec<ModuleLifecycleTransitionToken>,
}

impl<'a> LifecycleTransactionSet<'a> {
    pub(super) fn new(handle: &'a CoreHandle, capacity: usize) -> Self {
        Self {
            handle,
            tokens: Vec::with_capacity(capacity),
        }
    }

    pub(super) fn push(&mut self, token: ModuleLifecycleTransitionToken) {
        self.tokens.push(token);
    }

    pub(super) fn finish(mut self, result: Result<(), CoreError>) -> Result<(), CoreError> {
        self.complete_all(result.clone());
        result
    }

    fn complete_all(&mut self, result: Result<(), CoreError>) {
        for token in &self.tokens {
            self.handle
                .complete_module_lifecycle_transition(token, result.clone());
        }
        self.tokens.clear();
    }
}

impl Drop for LifecycleTransactionSet<'_> {
    fn drop(&mut self) {
        if self.tokens.is_empty() {
            return;
        }
        self.complete_all(Err(CoreError::ModuleLifecycleCallbackPanicked {
            module: "batch activation".to_owned(),
            command: ModuleLifecycleCommand::Activate.as_str(),
        }));
    }
}
