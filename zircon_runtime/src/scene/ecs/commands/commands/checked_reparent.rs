use crate::scene::ecs::{
    Command, DeferredCommandError, DeferredCommandOperation, DeferredCommandTarget,
};
use crate::scene::{EntityId, World};

pub(super) struct CheckedReparentCommand {
    child: EntityId,
    parent: Option<EntityId>,
}

impl CheckedReparentCommand {
    pub(super) fn new(child: EntityId, parent: Option<EntityId>) -> Self {
        Self { child, parent }
    }
}

impl Command for CheckedReparentCommand {
    fn apply(self, world: &mut World) {
        let owner = world.owner_identity().clone();
        if let Err(error) = world.set_parent_checked(self.child, self.parent) {
            if !owner.is_same_world(world.owner_identity()) {
                return;
            }
            world.record_deferred_command_error(DeferredCommandError::new(
                DeferredCommandOperation::ReparentChecked,
                DeferredCommandTarget::resolved(self.child),
                error,
            ));
        }
    }
}
