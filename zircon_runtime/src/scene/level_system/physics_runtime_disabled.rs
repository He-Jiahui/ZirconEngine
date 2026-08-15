use crate::scene::world::World;

pub(super) fn clear_retained_pose_resources(_world: &mut World) {}

#[derive(Clone, Debug, Default)]
pub(super) struct PhysicsRuntimeState;

impl PhysicsRuntimeState {
    pub(super) fn reset_after_world_replacement(&mut self) {}
}
