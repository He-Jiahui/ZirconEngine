use std::collections::BTreeMap;

use crate::scene::EntityId;
use crate::scene::ecs::{
    ChangeTick, ComponentId, ComponentTicks, component::TableColumnLayout, storage::StoredComponent,
};

use super::id::ArchetypeId;
use super::signature::ArchetypeSignature;
use super::table::{
    ArchetypePreflightedRow, ArchetypeTable, ArchetypeTableError, ArchetypeTakenRow,
};

#[derive(Debug)]
pub struct ArchetypeRecord {
    id: ArchetypeId,
    signature: ArchetypeSignature,
    table: ArchetypeTable,
    membership_generation: u64,
}

impl ArchetypeRecord {
    pub(super) fn new(
        id: ArchetypeId,
        signature: ArchetypeSignature,
        table_columns: impl IntoIterator<Item = (ComponentId, TableColumnLayout)>,
    ) -> Self {
        Self {
            id,
            signature,
            table: ArchetypeTable::new(table_columns),
            membership_generation: 0,
        }
    }

    pub fn id(&self) -> ArchetypeId {
        self.id
    }

    pub fn signature(&self) -> &ArchetypeSignature {
        &self.signature
    }

    pub fn entities(&self) -> &[EntityId] {
        self.table.entities()
    }

    /// Advances only when this archetype's row membership changes.
    ///
    /// Query plans use this instead of a world-global revision so a structural
    /// change in an unrelated archetype does not rebuild their cached rows.
    pub fn membership_generation(&self) -> u64 {
        self.membership_generation
    }

    pub(crate) fn estimated_heap_bytes(&self) -> usize {
        self.signature
            .estimated_heap_bytes()
            .saturating_add(self.table.estimated_heap_bytes())
    }

    pub(super) fn preflight_row(
        &self,
        components: impl IntoIterator<Item = (ComponentId, StoredComponent, ComponentTicks)>,
    ) -> Result<ArchetypePreflightedRow, ArchetypeTableError> {
        self.table.preflight_row(components)
    }

    pub(super) fn validate_row_components(
        &self,
        components: &BTreeMap<ComponentId, (StoredComponent, ComponentTicks)>,
    ) -> Result<(), ArchetypeTableError> {
        self.table.validate_row_components(components)
    }

    pub(super) fn validate_transition(
        &self,
        source_component_ids: impl IntoIterator<Item = ComponentId>,
        updates: &BTreeMap<ComponentId, Option<(StoredComponent, ComponentTicks)>>,
    ) -> Result<(), ArchetypeTableError> {
        self.table
            .validate_transition(source_component_ids, updates)
    }

    pub(super) fn bind_prevalidated_row(
        &self,
        components: BTreeMap<ComponentId, (StoredComponent, ComponentTicks)>,
    ) -> ArchetypePreflightedRow {
        self.table.bind_prevalidated_row(components)
    }

    pub(super) fn append_preflighted_row(
        &mut self,
        entity: EntityId,
        row: ArchetypePreflightedRow,
    ) -> usize {
        let row = self.table.append_preflighted_row(entity, row);
        self.membership_generation = self.membership_generation.wrapping_add(1);
        row
    }

    pub(super) fn take_row(
        &mut self,
        row: usize,
        entity: EntityId,
    ) -> Result<ArchetypeTakenRow, ArchetypeTableError> {
        let row = self.table.take_row(row, entity)?;
        self.membership_generation = self.membership_generation.wrapping_add(1);
        Ok(row)
    }

    pub(super) fn get<T>(&self, component_id: ComponentId, row: usize) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        self.table.get(component_id, row)
    }

    pub(super) fn column_slot(&self, component_id: ComponentId) -> Option<usize> {
        self.table.column_slot(component_id)
    }

    pub(super) fn get_by_slot<T>(&self, column_slot: usize, row: usize) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        self.table.get_by_slot(column_slot, row)
    }

    pub(super) fn component_ticks_by_slot(
        &self,
        column_slot: usize,
        row: usize,
    ) -> Option<ComponentTicks> {
        self.table.component_ticks_by_slot(column_slot, row)
    }

    pub(super) fn get_mut_at_tick_by_slot<T>(
        &mut self,
        column_slot: usize,
        row: usize,
        tick: ChangeTick,
    ) -> Option<&mut T>
    where
        T: Send + Sync + 'static,
    {
        self.table.get_mut_at_tick_by_slot(column_slot, row, tick)
    }

    pub(super) fn get_mut_with_ticks_by_slot<T>(
        &mut self,
        column_slot: usize,
        row: usize,
    ) -> Option<(&mut T, &mut ComponentTicks)>
    where
        T: Send + Sync + 'static,
    {
        self.table.get_mut_with_ticks_by_slot(column_slot, row)
    }

    pub(super) fn get_mut_at_tick<T>(
        &mut self,
        component_id: ComponentId,
        row: usize,
        tick: ChangeTick,
    ) -> Option<&mut T>
    where
        T: Send + Sync + 'static,
    {
        self.table.get_mut_at_tick(component_id, row, tick)
    }

    pub(super) fn get_mut_with_ticks<T>(
        &mut self,
        component_id: ComponentId,
        row: usize,
    ) -> Option<(&mut T, &mut ComponentTicks)>
    where
        T: Send + Sync + 'static,
    {
        self.table.get_mut_with_ticks(component_id, row)
    }

    pub(super) fn component_ticks(
        &self,
        component_id: ComponentId,
        row: usize,
    ) -> Option<ComponentTicks> {
        self.table.component_ticks(component_id, row)
    }

    pub(super) fn replace(
        &mut self,
        component_id: ComponentId,
        row: usize,
        value: StoredComponent,
        tick: ChangeTick,
    ) -> Option<StoredComponent> {
        self.table.replace(component_id, row, value, tick)
    }

    pub(super) fn for_each_component<T>(
        &self,
        component_id: ComponentId,
        visit: impl FnMut(EntityId, &T),
    ) where
        T: Send + Sync + 'static,
    {
        self.table.for_each_component(component_id, visit);
    }
}
