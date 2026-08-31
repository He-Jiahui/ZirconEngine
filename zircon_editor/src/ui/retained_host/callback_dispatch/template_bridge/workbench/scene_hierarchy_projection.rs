use std::collections::{BTreeMap, BTreeSet, HashMap};

use zircon_runtime::scene::{EntityId, WorldInspectionHierarchyRow};

/// Retained-only lookup state derived from a complete hierarchy reflow.
///
/// Runtime rows remain authoritative. This owner stores compact logical row/index state, bounded
/// authored-control routing, and the editor-owned selection overlay so sparse messages never need
/// an O(N) scan of a materialized hierarchy.
#[derive(Debug, Default)]
pub(super) struct SceneHierarchyProjectionState {
    generation: Option<u64>,
    selection_revision: Option<u64>,
    rows_by_entity: HashMap<EntityId, SceneHierarchyRowState>,
    controls_by_entity: BTreeMap<EntityId, String>,
    entities_by_control: BTreeMap<String, EntityId>,
    selected_entities: BTreeSet<EntityId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SceneHierarchyRowState {
    row_index: usize,
    parent: Option<EntityId>,
    depth: u32,
}

impl SceneHierarchyRowState {
    fn from_row(row_index: usize, row: &WorldInspectionHierarchyRow) -> Self {
        Self {
            row_index,
            parent: row.parent,
            depth: row.depth,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SceneHierarchyLogicalRowPatch {
    row_index: usize,
    replacement: Option<SceneHierarchyLogicalRowContent>,
    selected: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SceneHierarchyLogicalRowContent {
    entity: EntityId,
    display_name: String,
    depth: u32,
}

impl SceneHierarchyLogicalRowPatch {
    pub(crate) const fn row_index(&self) -> usize {
        self.row_index
    }

    pub(crate) const fn replacement(&self) -> Option<&SceneHierarchyLogicalRowContent> {
        self.replacement.as_ref()
    }

    pub(crate) const fn selected(&self) -> bool {
        self.selected
    }
}

impl SceneHierarchyLogicalRowContent {
    pub(crate) const fn entity(&self) -> EntityId {
        self.entity
    }

    pub(crate) fn display_name(&self) -> &str {
        &self.display_name
    }

    pub(crate) const fn depth(&self) -> u32 {
        self.depth
    }
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
        self.rows_by_entity = rows
            .iter()
            .enumerate()
            .map(|(row_index, row)| (row.entity, SceneHierarchyRowState::from_row(row_index, row)))
            .collect();
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

    pub(super) fn contains_entity(&self, entity: EntityId) -> bool {
        self.rows_by_entity.contains_key(&entity)
    }

    pub(super) fn row_identity_matches(&self, row: &WorldInspectionHierarchyRow) -> bool {
        self.rows_by_entity
            .get(&row.entity)
            .is_some_and(|current| current.parent == row.parent && current.depth == row.depth)
    }

    pub(super) fn patch_row(&mut self, row: &WorldInspectionHierarchyRow) {
        if let Some(current) = self.rows_by_entity.get_mut(&row.entity) {
            current.parent = row.parent;
            current.depth = row.depth;
        }
    }

    pub(super) fn logical_selection_patch(
        &self,
        entity: EntityId,
    ) -> Option<SceneHierarchyLogicalRowPatch> {
        let row = self.rows_by_entity.get(&entity)?;
        Some(SceneHierarchyLogicalRowPatch {
            row_index: row.row_index,
            replacement: None,
            selected: self.is_selected(entity),
        })
    }

    pub(super) fn logical_content_patch(
        &self,
        row: &WorldInspectionHierarchyRow,
    ) -> Option<SceneHierarchyLogicalRowPatch> {
        let current = self.rows_by_entity.get(&row.entity)?;
        Some(SceneHierarchyLogicalRowPatch {
            row_index: current.row_index,
            replacement: Some(SceneHierarchyLogicalRowContent {
                entity: row.entity,
                display_name: row.display_name.clone(),
                depth: row.depth,
            }),
            selected: self.is_selected(row.entity),
        })
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
