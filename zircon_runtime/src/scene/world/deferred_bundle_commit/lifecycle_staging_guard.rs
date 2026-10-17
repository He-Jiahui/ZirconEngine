use crate::scene::world::DetachPreparationOwner;
use crate::scene::World;

/// Keeps World exclusively borrowed until lifecycle staging is restored.
/// Publication reborrows World through this guard, before lifecycle callbacks
/// replay; no stored row/column reference crosses that boundary.
pub(super) struct LifecycleStagingGuard<'world> {
    world: &'world mut World,
    owner: DetachPreparationOwner,
    previous: bool,
    event_start: usize,
}

impl<'world> LifecycleStagingGuard<'world> {
    pub(super) fn new(world: &'world mut World, previous: bool, event_start: usize) -> Self {
        let owner = world.owner_identity().clone();
        Self {
            world,
            owner,
            previous,
            event_start,
        }
    }

    pub(super) fn world(&mut self) -> &mut World {
        &mut *self.world
    }
}

impl Drop for LifecycleStagingGuard<'_> {
    fn drop(&mut self) {
        let world = &mut *self.world;
        if self.owner.is_same_world(world.owner_identity()) {
            world.record_staged_lifecycle_events = self.previous;
            if std::thread::panicking() {
                // Partially published values remain in the original World.
                // Unpublished staged event records are owned strings/IDs.
                world.staged_lifecycle_events.truncate(self.event_start);
            }
        }
    }
}
