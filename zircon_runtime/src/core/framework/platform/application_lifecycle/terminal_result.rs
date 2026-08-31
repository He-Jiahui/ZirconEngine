use super::{ApplicationLifecycleOperationId, ApplicationLifecycleState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApplicationLifecycleTerminalResult {
    operation: ApplicationLifecycleOperationId,
    state: ApplicationLifecycleState,
}

impl ApplicationLifecycleTerminalResult {
    pub(crate) const fn new(
        operation: ApplicationLifecycleOperationId,
        state: ApplicationLifecycleState,
    ) -> Self {
        Self { operation, state }
    }

    pub const fn operation(self) -> ApplicationLifecycleOperationId {
        self.operation
    }

    pub const fn state(self) -> ApplicationLifecycleState {
        self.state
    }
}
