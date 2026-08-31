use std::sync::Arc;

use crate::scene::EntityId;
use crate::scene::ecs::{
    ChangeTick, Component, ComponentMutationRecord, ComponentMutationRecorder,
    ComponentMutationSink,
};

use super::{RenderDirtyEntityJournal, RenderDirtyWorldId};

#[derive(Clone, Debug)]
pub(in crate::scene::world) struct RenderDirtyJournalState {
    world: RenderDirtyWorldId,
    generation: u64,
    pending_all: bool,
    pending_entities: Vec<EntityId>,
    component_mutations: ComponentMutationSink,
    published: Arc<RenderDirtyEntityJournal>,
}

impl PartialEq for RenderDirtyJournalState {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for RenderDirtyJournalState {}

impl RenderDirtyJournalState {
    pub(in crate::scene::world) fn all() -> Self {
        let world = RenderDirtyWorldId::allocate();
        Self {
            world,
            generation: 0,
            pending_all: true,
            pending_entities: Vec::new(),
            component_mutations: ComponentMutationSink::default(),
            published: Arc::new(RenderDirtyEntityJournal::empty(world)),
        }
    }

    pub(in crate::scene::world) fn mark_all(&mut self) {
        self.pending_all = true;
        self.pending_entities.clear();
    }

    pub(in crate::scene::world) fn mark(&mut self, entity: EntityId) {
        if !self.pending_all {
            self.pending_entities.push(entity);
        }
    }

    pub(in crate::scene::world) fn has_pending(&self) -> bool {
        self.pending_all
            || !self.pending_entities.is_empty()
            || self.component_mutations.pending_count() != 0
    }

    pub(in crate::scene::world) fn pending_component_mutation_count(&self) -> u64 {
        self.component_mutations.pending_count()
    }

    pub(in crate::scene::world) fn component_mutation_recorder<T>(
        &self,
        entity: EntityId,
    ) -> ComponentMutationRecorder<'_>
    where
        T: Component,
    {
        self.component_mutations.recorder::<T>(entity)
    }

    pub(in crate::scene::world) fn take_component_mutations(&self) -> Vec<ComponentMutationRecord> {
        self.component_mutations.drain()
    }

    pub(in crate::scene::world) fn publish(
        &mut self,
        source_world_generation: u64,
        source_change_tick: ChangeTick,
    ) {
        if !self.pending_all && self.pending_entities.is_empty() {
            return;
        }

        let all_entities = self.pending_all;
        let mut entities = std::mem::take(&mut self.pending_entities);
        if all_entities {
            entities.clear();
        } else {
            entities.sort_unstable();
            entities.dedup();
        }
        self.generation = self
            .generation
            .checked_add(1)
            .expect("render-dirty journal generation exhausted");
        self.published = Arc::new(RenderDirtyEntityJournal::new(
            self.world,
            self.generation,
            source_world_generation,
            source_change_tick,
            all_entities,
            entities,
        ));
        self.pending_all = false;
    }

    pub(in crate::scene::world) fn published(&self) -> Arc<RenderDirtyEntityJournal> {
        Arc::clone(&self.published)
    }
}
