use std::sync::{LockResult, Mutex, MutexGuard};

/// Serializes the complete manager-owned project activation and close transactions.
///
/// Session admission, runtime effects, Ready publication, close compensation, and final lease
/// release form one lifecycle transaction. Serializing only the retained `SessionGuard` slot would
/// allow two activations, or an activation and close, to mutate the runtime concurrently.
#[derive(Debug, Default)]
pub(super) struct ProjectSessionTransitionGate {
    transition: Mutex<()>,
}

impl ProjectSessionTransitionGate {
    pub(super) fn enter(&self) -> LockResult<MutexGuard<'_, ()>> {
        self.transition.lock()
    }
}

#[cfg(test)]
#[path = "tests/project_session_transition.rs"]
mod tests;
