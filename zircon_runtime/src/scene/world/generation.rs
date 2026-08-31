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

    pub(super) const fn advanced_by(self, count: u64) -> Self {
        Self(self.0.saturating_add(count))
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

/// Runtime-only revision published with component lifecycle visibility.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct LifecycleVisibilityRevision(u64);

impl LifecycleVisibilityRevision {
    pub(super) const fn get(self) -> u64 {
        self.0
    }

    pub(super) fn advance(&mut self) {
        self.0 = self.0.saturating_add(1);
    }
}

// Runtime revisions do not participate in persistent world equality.
impl PartialEq for LifecycleVisibilityRevision {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl World {
    /// Returns the current runtime synchronization revision.
    pub fn world_generation(&self) -> u64 {
        self.world_generation
            .get()
            .saturating_add(self.derived_state_dirty.pending_component_mutation_count())
    }

    /// Returns the revision observed by component lifecycle callbacks.
    pub(crate) fn lifecycle_visibility_revision(&self) -> u64 {
        self.lifecycle_visibility_revision.get()
    }

    /// Returns the revision for one dynamic component type only.
    ///
    /// Consumers that project a single component can avoid rebuilding after
    /// unrelated world mutations such as transform updates.
    pub fn dynamic_component_generation(&self, component_id: &str) -> u64 {
        self.dynamic_component_generations
            .get(component_id)
            .copied()
            .unwrap_or_default()
    }

    pub(super) fn advance_world_generation(&mut self) {
        self.world_generation.advance();
    }

    pub(super) fn bump_lifecycle_visibility_revision(&mut self) {
        self.lifecycle_visibility_revision.advance();
    }

    pub(super) fn advance_dynamic_component_generation(&mut self, component_id: &str) {
        let generation = self
            .dynamic_component_generations
            .entry(component_id.to_string())
            .or_default();
        *generation = generation.saturating_add(1);
    }

    /// Carries component-specific projection revisions across whole-world replacement.
    ///
    /// A deserialized world intentionally starts with no runtime revisions. Replacement must still
    /// invalidate a cached projection for both removed and newly introduced component types.
    pub(in crate::scene) fn advance_dynamic_component_generations_after(
        &mut self,
        previous: &Self,
    ) {
        let mut component_ids = previous
            .dynamic_component_generations
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        component_ids.extend(
            previous
                .dynamic_components
                .values()
                .flat_map(|components| components.keys().cloned()),
        );
        component_ids.extend(self.dynamic_component_generations.keys().cloned());
        component_ids.extend(
            self.dynamic_components
                .values()
                .flat_map(|components| components.keys().cloned()),
        );
        component_ids.sort_unstable();
        component_ids.dedup();

        for component_id in component_ids {
            let prior = previous.dynamic_component_generation(&component_id);
            let staged = self.dynamic_component_generation(&component_id);
            self.dynamic_component_generations
                .insert(component_id, prior.max(staged).saturating_add(1));
        }
    }

    pub(in crate::scene) fn advance_world_generation_after(&mut self, previous: u64) {
        self.world_generation
            .advance_after(WorldGeneration(previous));
    }
}

#[cfg(test)]
mod tests;
