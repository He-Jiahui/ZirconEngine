use super::{ApplicationLifecycleOperationId, ApplicationLifecycleState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApplicationLifecycleOperation {
    id: ApplicationLifecycleOperationId,
    target: ApplicationLifecycleState,
}

impl ApplicationLifecycleOperation {
    pub(crate) const fn new(
        id: ApplicationLifecycleOperationId,
        target: ApplicationLifecycleState,
    ) -> Self {
        Self { id, target }
    }

    pub const fn id(self) -> ApplicationLifecycleOperationId {
        self.id
    }

    pub const fn target(self) -> ApplicationLifecycleState {
        self.target
    }
}
