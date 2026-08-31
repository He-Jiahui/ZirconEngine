use super::state::ApplicationLifecycleState;

#[derive(Debug, Default)]
pub(super) struct ApplicationLifecycleMachine {
    pub(super) state: ApplicationLifecycleState,
}
