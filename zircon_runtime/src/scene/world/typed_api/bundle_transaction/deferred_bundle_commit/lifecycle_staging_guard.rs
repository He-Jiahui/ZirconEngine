use crate::scene::world::DetachPreparationOwner;
use crate::scene::World;

/// The owning transaction keeps this World container exclusively borrowed and
/// alive until this guard is dropped, before any lifecycle callback is replayed.
/// No stored row/column reference crosses that replay boundary.
pub(super) struct LifecycleStagingGuard {
    world: *mut World,
    owner: DetachPreparationOwner,
    previous: bool,
    event_start: usize,
}

impl LifecycleStagingGuard {
    pub(super) fn new(world: &mut World, previous: bool, event_start: usize) -> Self {
        let owner = world.owner_identity().clone();
        Self {
            world,
            owner,
            previous,
            event_start,
        }
    }
}

impl Drop for LifecycleStagingGuard {
    fn drop(&mut self) {
        // SAFETY: the enclosing transaction retains the original exclusive
        // World-container borrow through this guard's entire lexical lifetime.
        // The guard runs before lifecycle dispatch and creates no retained loan.
        let world = unsafe { &mut *self.world };
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
