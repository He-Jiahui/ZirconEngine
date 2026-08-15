use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::scene::{EntityId, WorldInspectionHierarchyRow};

/// Retained-only lookup state derived from a complete hierarchy reflow.
///
/// Runtime rows remain authoritative. This owner stores only control routing and the editor-owned
/// selection overlay so sparse messages never need an O(N) scan of a materialized hierarchy.
#[derive(Debug, Default)]
pub(super) struct SceneHierarchyProjectionState {
    generation: Option<u64>,
    selection_revision: Option<u64>,
    controls_by_entity: BTreeMap<EntityId, String>,
    entities_by_control: BTreeMap<String, EntityId>,
    selected_entities: BTreeSet<EntityId>,
}

impl SceneHierarchyProjectionState {
    pub(super) fn replace(
        &mut self,
        generation: Option<u64>,
        selection_revision: Option<u64>,
        rows: &[WorldInspectionHierarchyRow],
        controls: &[String],
        selected_entities: &BTreeSet<EntityId>,
    ) {
        self.generation = generation;
        self.selection_revision = selection_revision;
        let controls_by_entity = rows
            .iter()
            .zip(controls)
            .map(|(row, control_id)| (row.entity, control_id.clone()))
            .collect::<BTreeMap<_, _>>();
        self.entities_by_control = controls_by_entity
            .iter()
            .map(|(entity, control_id)| (control_id.clone(), *entity))
            .collect();
        self.controls_by_entity = controls_by_entity;
        self.selected_entities.clone_from(selected_entities);
    }

    pub(super) const fn generation(&self) -> Option<u64> {
        self.generation
    }

    pub(super) fn replace_generation(&mut self, generation: Option<u64>) {
        self.generation = generation;
    }

    pub(super) const fn selection_revision(&self) -> Option<u64> {
        self.selection_revision
    }

    pub(super) fn replace_selection_revision(&mut self, selection_revision: Option<u64>) {
        self.selection_revision = selection_revision;
    }

    pub(super) fn control_for(&self, entity: EntityId) -> Option<&str> {
        self.controls_by_entity.get(&entity).map(String::as_str)
    }

    pub(super) fn contains_control(&self, control_id: &str) -> bool {
        self.entities_by_control.contains_key(control_id)
    }

    pub(super) fn entity_for_control(&self, control_id: &str) -> Option<EntityId> {
        self.entities_by_control.get(control_id).copied()
    }

    pub(super) fn is_selected(&self, entity: EntityId) -> bool {
        self.selected_entities.contains(&entity)
    }

    pub(super) fn select(&mut self, entity: EntityId) {
        self.selected_entities.insert(entity);
    }

    pub(super) fn deselect(&mut self, entity: EntityId) {
        self.selected_entities.remove(&entity);
    }

    pub(super) fn selected_entities(&self) -> &BTreeSet<EntityId> {
        &self.selected_entities
    }

    pub(super) fn replace_selected_entities(&mut self, selected_entities: BTreeSet<EntityId>) {
        self.selected_entities = selected_entities;
    }
}
