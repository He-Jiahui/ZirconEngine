use super::World;

/// Runtime-only monotonic revision for observable world mutations.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct WorldGeneration(u64);

impl WorldGeneration {
    pub(super) const fn get(self) -> u64 {
        self.0
    }

    fn advance(&mut self) {
        self.0 = self.0.saturating_add(1);
    }

    /// Carries a session revision across wholesale world replacement.
    fn advance_after(&mut self, previous: Self) {
        self.0 = self.0.max(previous.0).saturating_add(1);
    }
}

// Runtime revisions do not participate in persistent world equality.
impl PartialEq for WorldGeneration {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl World {
    /// Returns the current runtime synchronization revision.
    pub fn world_generation(&self) -> u64 {
        self.world_generation.get()
    }

    pub(super) fn advance_world_generation(&mut self) {
        self.world_generation.advance();
    }

    pub(in crate::scene) fn advance_world_generation_after(&mut self, previous: u64) {
        self.world_generation
            .advance_after(WorldGeneration(previous));
    }
}

#[cfg(test)]
mod tests;
