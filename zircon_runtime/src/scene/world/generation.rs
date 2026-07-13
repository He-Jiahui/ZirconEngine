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
}

#[cfg(test)]
mod tests;
