use std::sync::Arc;

use crate::scene::ecs::{
    ChangeTick, Component, ComponentMutationRecord, ComponentMutationRecorder,
    ComponentMutationSink,
};
use crate::scene::EntityId;

use super::{RenderDirtyEntityJournal, RenderDirtyWorldId};

/// World 内部累积待发布实体和组件变更；publish 只在边界生成新的不可变 journal。
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
        unsafe { Self::mark_unchecked(self, entity) }
    }

    /// # Safety
    /// The caller must exclusively own the pending fields. Only the disjoint
    /// component-mutation sink may be retained by live query items, not this state.
    pub(in crate::scene::world) unsafe fn mark_unchecked(state: *mut Self, entity: EntityId) {
        unsafe {
            if !std::ptr::addr_of!((*state).pending_all).read() {
                (&mut *std::ptr::addr_of_mut!((*state).pending_entities)).push(entity);
            }
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

    /// # Safety
    /// The sink leaf must remain allocated for `'world`. The owner must keep this
    /// state fixed and prevent exclusive parent/sink access while the recorder lives.
    pub(in crate::scene::world) unsafe fn component_mutation_recorder_unchecked<'world, T>(
        state: *const Self,
        entity: EntityId,
    ) -> ComponentMutationRecorder<'world>
    where
        T: Component,
    {
        unsafe {
            let sink: &'world ComponentMutationSink =
                &*std::ptr::addr_of!((*state).component_mutations);
            sink.recorder::<T>(entity)
        }
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
