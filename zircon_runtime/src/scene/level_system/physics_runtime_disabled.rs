#[derive(Clone, Debug, Default)]
pub(super) struct PhysicsRuntimeState;

impl PhysicsRuntimeState {
    pub(super) fn reset_after_world_replacement(&mut self) {}
}
