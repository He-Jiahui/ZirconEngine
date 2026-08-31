use std::collections::BTreeSet;
use std::sync::Arc;

use crate::core::framework::render::RenderComponentChangeArtifact;
use crate::scene::EntityId;
use crate::scene::ecs::{
    ChangeTick, Component, ComponentMutationRecord, ComponentMutationRecorder, InternalSceneSystem,
};

use super::render_component_changes::RenderComponentChangeProjector;
use super::render_dirty_journal::{RenderDirtyEntityJournal, RenderDirtyJournalState};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct DerivedStateFrontier {
    all: bool,
    entities: BTreeSet<EntityId>,
}

impl DerivedStateFrontier {
    fn all() -> Self {
        Self {
            all: true,
            entities: BTreeSet::new(),
        }
    }

    fn mark(&mut self, entity: EntityId) {
        if !self.all {
            self.entities.insert(entity);
        }
    }

    pub(super) fn contains(&self, entity: EntityId) -> bool {
        self.all || self.entities.contains(&entity)
    }

    pub(super) const fn is_all(&self) -> bool {
        self.all
    }

    pub(super) fn entities(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.entities.iter().copied()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DerivedStateDirty {
    hierarchy: bool,
    active: DerivedStateFrontier,
    transforms: DerivedStateFrontier,
    node_cache: DerivedStateFrontier,
    render_extract: RenderDirtyJournalState,
    render_component_changes: RenderComponentChangeProjector,
    defer_flush: bool,
}

impl Default for DerivedStateDirty {
    fn default() -> Self {
        Self::all()
    }
}

impl DerivedStateDirty {
    pub(super) fn all() -> Self {
        Self {
            hierarchy: true,
            active: DerivedStateFrontier::all(),
            transforms: DerivedStateFrontier::all(),
            node_cache: DerivedStateFrontier::all(),
            render_extract: RenderDirtyJournalState::all(),
            render_component_changes: RenderComponentChangeProjector::default(),
            defer_flush: false,
        }
    }

    pub(super) fn mark_hierarchy(&mut self) {
        self.hierarchy = true;
        self.active = DerivedStateFrontier::all();
        self.transforms = DerivedStateFrontier::all();
        self.node_cache = DerivedStateFrontier::all();
        self.render_extract.mark_all();
    }

    pub(super) fn mark_hierarchy_at(&mut self, entity: EntityId) {
        self.hierarchy = true;
        self.active.mark(entity);
        self.transforms.mark(entity);
        self.node_cache.mark(entity);
        self.render_extract.mark(entity);
    }

    pub(super) fn mark_checked_hierarchy_at(&mut self, entity: EntityId) {
        self.active.mark(entity);
        self.transforms.mark(entity);
        self.node_cache.mark(entity);
        self.render_extract.mark(entity);
    }

    pub(super) fn mark_hierarchy_repaired(&mut self) {
        self.active = DerivedStateFrontier::all();
        self.transforms = DerivedStateFrontier::all();
        self.node_cache = DerivedStateFrontier::all();
        self.render_extract.mark_all();
    }

    pub(super) fn mark_active(&mut self) {
        self.active = DerivedStateFrontier::all();
        self.node_cache = DerivedStateFrontier::all();
        self.render_extract.mark_all();
    }

    pub(super) fn mark_active_at(&mut self, entity: EntityId) {
        self.active.mark(entity);
        self.node_cache.mark(entity);
        self.render_extract.mark(entity);
    }

    pub(super) fn mark_transform(&mut self) {
        self.transforms = DerivedStateFrontier::all();
        self.node_cache = DerivedStateFrontier::all();
        self.render_extract.mark_all();
    }

    pub(super) fn mark_transform_at(&mut self, entity: EntityId) {
        self.transforms.mark(entity);
        self.node_cache.mark(entity);
        self.render_extract.mark(entity);
    }

    pub(super) fn mark_node_cache(&mut self) {
        self.node_cache = DerivedStateFrontier::all();
        self.render_extract.mark_all();
    }

    pub(super) fn mark_node_cache_at(&mut self, entity: EntityId) {
        self.node_cache.mark(entity);
        self.render_extract.mark(entity);
    }

    pub(super) fn should_run(&self, system: InternalSceneSystem) -> bool {
        match system {
            InternalSceneSystem::ApplyDeferred => false,
            InternalSceneSystem::UpdateEvents => false,
            InternalSceneSystem::HierarchyValidity => self.hierarchy,
            InternalSceneSystem::ActiveHierarchy => {
                self.active.all || !self.active.entities.is_empty()
            }
            InternalSceneSystem::WorldTransform => {
                self.transforms.all || !self.transforms.entities.is_empty()
            }
            InternalSceneSystem::NodeCache => {
                self.node_cache.all || !self.node_cache.entities.is_empty()
            }
            InternalSceneSystem::RenderExtractPrepare => self.render_extract.has_pending(),
        }
    }

    pub(super) fn clear(&mut self, system: InternalSceneSystem) {
        match system {
            InternalSceneSystem::ApplyDeferred => {}
            InternalSceneSystem::UpdateEvents => {}
            InternalSceneSystem::HierarchyValidity => self.hierarchy = false,
            InternalSceneSystem::ActiveHierarchy => self.active = DerivedStateFrontier::default(),
            InternalSceneSystem::WorldTransform => {
                self.transforms = DerivedStateFrontier::default()
            }
            InternalSceneSystem::NodeCache => self.node_cache = DerivedStateFrontier::default(),
            InternalSceneSystem::RenderExtractPrepare => {}
        }
    }

    pub(super) fn set_defer_flush(&mut self, defer_flush: bool) {
        self.defer_flush = defer_flush;
    }

    pub(super) fn hierarchy_or_transform_pending(&self) -> bool {
        self.hierarchy
            || self.transforms.all
            || !self.transforms.entities.is_empty()
            || self.render_extract.pending_component_mutation_count() != 0
    }

    pub(super) fn active_pending(&self) -> bool {
        self.hierarchy
            || self.active.all
            || !self.active.entities.is_empty()
            || self.render_extract.pending_component_mutation_count() != 0
    }

    pub(super) fn has_pending(&self) -> bool {
        self.hierarchy
            || self.active.all
            || !self.active.entities.is_empty()
            || self.transforms.all
            || !self.transforms.entities.is_empty()
            || self.node_cache.all
            || !self.node_cache.entities.is_empty()
            || self.render_extract.has_pending()
    }

    pub(super) fn take_active_frontier(&mut self) -> DerivedStateFrontier {
        std::mem::take(&mut self.active)
    }

    pub(super) fn take_transform_frontier(&mut self) -> DerivedStateFrontier {
        std::mem::take(&mut self.transforms)
    }

    pub(super) fn take_node_cache_frontier(&mut self) -> DerivedStateFrontier {
        std::mem::take(&mut self.node_cache)
    }

    pub(super) fn mark_render_dirty_at(&mut self, entity: EntityId) {
        self.render_extract.mark(entity);
    }

    pub(super) fn mark_render_dirty(&mut self) {
        self.render_extract.mark_all();
    }

    pub(super) fn component_mutation_recorder<T>(
        &self,
        entity: EntityId,
    ) -> ComponentMutationRecorder<'_>
    where
        T: Component,
    {
        self.render_extract.component_mutation_recorder::<T>(entity)
    }

    pub(super) fn take_component_mutations(&self) -> Vec<ComponentMutationRecord> {
        self.render_extract.take_component_mutations()
    }

    pub(super) fn pending_component_mutation_count(&self) -> u64 {
        self.render_extract.pending_component_mutation_count()
    }

    pub(super) fn publish_render_dirty_journal(
        &mut self,
        source_world_generation: u64,
        source_change_tick: ChangeTick,
    ) {
        self.render_extract
            .publish(source_world_generation, source_change_tick);
    }

    pub(super) fn render_dirty_entity_journal(&self) -> Arc<RenderDirtyEntityJournal> {
        self.render_extract.published()
    }

    pub(super) fn take_render_component_change_projector(
        &mut self,
    ) -> RenderComponentChangeProjector {
        std::mem::take(&mut self.render_component_changes)
    }

    pub(super) fn restore_render_component_change_projector(
        &mut self,
        projector: RenderComponentChangeProjector,
    ) {
        self.render_component_changes = projector;
    }

    pub(super) fn render_component_change_artifact(
        &self,
    ) -> Option<Arc<RenderComponentChangeArtifact>> {
        self.render_component_changes.published()
    }
}
