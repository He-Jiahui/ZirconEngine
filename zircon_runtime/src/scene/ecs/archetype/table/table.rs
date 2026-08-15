use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::scene::ecs::{
    component::TableColumnLayout, storage::StoredComponent, ChangeTick, ComponentId, ComponentTicks,
};
use crate::scene::EntityId;

use super::column::ArchetypeColumn;
use super::error::ArchetypeTableError;
use super::preflighted_row::ArchetypePreflightedRow;
use super::taken_row::ArchetypeTakenRow;

/// Owns one archetype's row-aligned table component columns and their change ticks.
pub(crate) struct ArchetypeTable {
    entities: Vec<EntityId>,
    columns: Vec<(ComponentId, ArchetypeColumn)>,
}

impl ArchetypeTable {
    pub(crate) fn new(
        component_columns: impl IntoIterator<Item = (ComponentId, TableColumnLayout)>,
    ) -> Self {
        let mut columns = component_columns.into_iter().collect::<Vec<_>>();
        columns.sort_unstable_by_key(|(component_id, _)| *component_id);
        columns.dedup_by_key(|(component_id, _)| *component_id);
        let mut dense_columns = Vec::with_capacity(columns.len());
        for (component_id, layout) in columns {
            dense_columns.push((component_id, ArchetypeColumn::new(layout)));
        }
        Self {
            entities: Vec::new(),
            columns: dense_columns,
        }
    }

    pub(crate) fn entities(&self) -> &[EntityId] {
        &self.entities
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub(crate) fn len(&self) -> usize {
        self.entities.len()
    }

    pub(crate) fn estimated_heap_bytes(&self) -> usize {
        let entity_bytes = self
            .entities
            .capacity()
            .saturating_mul(std::mem::size_of::<EntityId>());
        let column_directory_bytes = self
            .columns
            .capacity()
            .saturating_mul(std::mem::size_of::<(ComponentId, ArchetypeColumn)>());
        self.columns.iter().fold(
            entity_bytes.saturating_add(column_directory_bytes),
            |bytes, (_, column)| bytes.saturating_add(column.estimated_heap_bytes()),
        )
    }

    pub(crate) fn component_ids(&self) -> impl ExactSizeIterator<Item = ComponentId> + '_ {
        self.columns.iter().map(|(component_id, _)| *component_id)
    }

    pub(crate) fn append_row(
        &mut self,
        entity: EntityId,
        components: impl IntoIterator<Item = (ComponentId, StoredComponent, ComponentTicks)>,
    ) -> Result<usize, ArchetypeTableError> {
        let row = self.preflight_row(components)?;
        Ok(self.append_preflighted_row(entity, row))
    }

    /// Validates a complete target row without making it observable in the table.
    pub(crate) fn preflight_row(
        &self,
        components: impl IntoIterator<Item = (ComponentId, StoredComponent, ComponentTicks)>,
    ) -> Result<ArchetypePreflightedRow, ArchetypeTableError> {
        let row = ArchetypePreflightedRow::collect(components)?;
        self.validate_row_components(row.components())?;
        Ok(row)
    }

    pub(crate) fn validate_row_components(
        &self,
        components: &BTreeMap<ComponentId, (StoredComponent, ComponentTicks)>,
    ) -> Result<(), ArchetypeTableError> {
        self.validate_row_components_inner(components)
    }

    /// Proves a structural row delta will produce this table's exact column
    /// set before the source row is removed from its current archetype.
    pub(crate) fn validate_transition(
        &self,
        source_component_ids: impl IntoIterator<Item = ComponentId>,
        updates: &BTreeMap<ComponentId, Option<(StoredComponent, ComponentTicks)>>,
    ) -> Result<(), ArchetypeTableError> {
        let mut final_component_ids = source_component_ids.into_iter().collect::<BTreeSet<_>>();
        for (component_id, update) in updates {
            match update {
                Some((value, _)) => {
                    let Some(column) = self.column(*component_id) else {
                        return Err(ArchetypeTableError::UnexpectedComponentColumn {
                            component_id: *component_id,
                        });
                    };
                    if !column.accepts(value) {
                        return Err(ArchetypeTableError::ComponentTypeMismatch {
                            component_id: *component_id,
                            expected_type: column.type_name(),
                        });
                    }
                    final_component_ids.insert(*component_id);
                }
                None => {
                    final_component_ids.remove(component_id);
                }
            }
        }

        for component_id in &final_component_ids {
            if self.column_slot(*component_id).is_none() {
                return Err(ArchetypeTableError::UnexpectedComponentColumn {
                    component_id: *component_id,
                });
            }
        }
        for (component_id, _) in &self.columns {
            if !final_component_ids.contains(component_id) {
                return Err(ArchetypeTableError::MissingComponentColumn {
                    component_id: *component_id,
                });
            }
        }
        Ok(())
    }

    pub(crate) fn bind_prevalidated_row(
        &self,
        components: BTreeMap<ComponentId, (StoredComponent, ComponentTicks)>,
    ) -> ArchetypePreflightedRow {
        debug_assert!(self.validate_row_components_inner(&components).is_ok());
        ArchetypePreflightedRow::from_validated_components(components)
    }

    /// Publishes values that [`ArchetypeTable::preflight_row`] already bound
    /// to this table's complete column set.
    pub(crate) fn append_preflighted_row(
        &mut self,
        entity: EntityId,
        row: ArchetypePreflightedRow,
    ) -> usize {
        let mut components = row.into_components();
        let row = self.entities.len();
        for (component_id, column) in &mut self.columns {
            let (value, ticks) = components
                .remove(&*component_id)
                .expect("validated archetype table row must initialize every column");
            column.push(value, ticks);
        }
        debug_assert!(components.is_empty());
        self.entities.push(entity);
        row
    }

    pub(crate) fn get<T>(&self, component_id: ComponentId, row: usize) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        self.column(component_id)?.get::<T>(row)
    }

    pub(crate) fn get_mut<T>(&mut self, component_id: ComponentId, row: usize) -> Option<&mut T>
    where
        T: Send + Sync + 'static,
    {
        self.column_mut(component_id)?.get_mut::<T>(row)
    }

    pub(crate) fn get_mut_at_tick<T>(
        &mut self,
        component_id: ComponentId,
        row: usize,
        tick: ChangeTick,
    ) -> Option<&mut T>
    where
        T: Send + Sync + 'static,
    {
        self.column_mut(component_id)?
            .get_mut_at_tick::<T>(row, tick)
    }

    pub(crate) fn get_mut_with_ticks<T>(
        &mut self,
        component_id: ComponentId,
        row: usize,
    ) -> Option<(&mut T, &mut ComponentTicks)>
    where
        T: Send + Sync + 'static,
    {
        self.column_mut(component_id)?.get_mut_with_ticks::<T>(row)
    }

    pub(crate) fn component_ticks(
        &self,
        component_id: ComponentId,
        row: usize,
    ) -> Option<ComponentTicks> {
        self.column(component_id)?.ticks(row)
    }

    /// Replaces one value in place, keeping the entity and every column row-aligned.
    pub(crate) fn replace(
        &mut self,
        component_id: ComponentId,
        row: usize,
        value: StoredComponent,
        tick: ChangeTick,
    ) -> Option<StoredComponent> {
        self.column_mut(component_id)?.replace(row, value, tick)
    }

    pub(crate) fn take_row(
        &mut self,
        row: usize,
        expected_entity: EntityId,
    ) -> Result<ArchetypeTakenRow, ArchetypeTableError> {
        let len = self.entities.len();
        if row >= len {
            return Err(ArchetypeTableError::RowOutOfBounds { row, len });
        }
        let actual_entity = self.entities[row];
        if actual_entity != expected_entity {
            return Err(ArchetypeTableError::EntityRowMismatch {
                row,
                expected: expected_entity,
                actual: actual_entity,
            });
        }

        let mut components = BTreeMap::new();
        for (component_id, column) in &mut self.columns {
            let component = column
                .take(row)
                .expect("every archetype table column must have the entity row");
            components.insert(*component_id, component);
        }
        let entity = self.entities.swap_remove(row);
        debug_assert_eq!(entity, expected_entity);
        let swapped_entity = (row < self.entities.len()).then(|| self.entities[row]);
        Ok(ArchetypeTakenRow::new(entity, swapped_entity, components))
    }

    pub(crate) fn for_each_component<T>(
        &self,
        component_id: ComponentId,
        mut visit: impl FnMut(EntityId, &T),
    ) where
        T: Send + Sync + 'static,
    {
        let Some(column) = self.column(component_id) else {
            return;
        };
        for (row, entity) in self.entities.iter().copied().enumerate() {
            let component = column
                .get::<T>(row)
                .expect("registered archetype column must keep one concrete value type");
            visit(entity, component);
        }
    }

    fn validate_row_components_inner(
        &self,
        components: &BTreeMap<ComponentId, (StoredComponent, ComponentTicks)>,
    ) -> Result<(), ArchetypeTableError> {
        for (component_id, (value, _)) in components {
            let Some(column) = self.column(*component_id) else {
                return Err(ArchetypeTableError::UnexpectedComponentColumn {
                    component_id: *component_id,
                });
            };
            if !column.accepts(value) {
                return Err(ArchetypeTableError::ComponentTypeMismatch {
                    component_id: *component_id,
                    expected_type: column.type_name(),
                });
            }
        }
        for (component_id, _) in &self.columns {
            if !components.contains_key(component_id) {
                return Err(ArchetypeTableError::MissingComponentColumn {
                    component_id: *component_id,
                });
            }
        }
        Ok(())
    }

    pub(crate) fn column_slot(&self, component_id: ComponentId) -> Option<usize> {
        self.columns
            .binary_search_by_key(&component_id, |(candidate, _)| *candidate)
            .ok()
    }

    pub(crate) fn get_by_slot<T>(&self, column_slot: usize, row: usize) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        self.columns.get(column_slot)?.1.get::<T>(row)
    }

    pub(crate) fn component_ticks_by_slot(
        &self,
        column_slot: usize,
        row: usize,
    ) -> Option<ComponentTicks> {
        self.columns.get(column_slot)?.1.ticks(row)
    }

    pub(crate) fn get_mut_at_tick_by_slot<T>(
        &mut self,
        column_slot: usize,
        row: usize,
        tick: ChangeTick,
    ) -> Option<&mut T>
    where
        T: Send + Sync + 'static,
    {
        self.columns
            .get_mut(column_slot)?
            .1
            .get_mut_at_tick::<T>(row, tick)
    }

    pub(crate) fn get_mut_with_ticks_by_slot<T>(
        &mut self,
        column_slot: usize,
        row: usize,
    ) -> Option<(&mut T, &mut ComponentTicks)>
    where
        T: Send + Sync + 'static,
    {
        self.columns
            .get_mut(column_slot)?
            .1
            .get_mut_with_ticks::<T>(row)
    }

    fn column(&self, component_id: ComponentId) -> Option<&ArchetypeColumn> {
        let slot = self.column_slot(component_id)?;
        Some(&self.columns[slot].1)
    }

    fn column_mut(&mut self, component_id: ComponentId) -> Option<&mut ArchetypeColumn> {
        let slot = self.column_slot(component_id)?;
        Some(&mut self.columns[slot].1)
    }
}

impl fmt::Debug for ArchetypeTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArchetypeTable")
            .field("entities", &self.entities)
            .field("component_ids", &self.component_ids().collect::<Vec<_>>())
            .finish()
    }
}
