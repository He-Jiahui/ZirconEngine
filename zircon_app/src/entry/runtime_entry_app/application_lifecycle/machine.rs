use super::state::ApplicationLifecycleState;

#[derive(Debug, Default)]
pub(in crate::entry::runtime_entry_app) struct ApplicationLifecycleMachine {
    pub(super) state: ApplicationLifecycleState,
}
