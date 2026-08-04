use std::collections::{btree_map::Entry, BTreeMap};

use crate::scene::ecs::{
    component::TableColumnLayout, storage::StoredComponent, ChangeTick, ComponentId, ComponentTicks,
};
use crate::scene::EntityId;

use super::column::ArchetypeColumn;
use super::error::ArchetypeTableError;
use super::taken_row::ArchetypeTakenRow;

/// Owns one archetype's row-aligned table component columns and their change ticks.
pub(crate) struct ArchetypeTable {
    entities: Vec<EntityId>,
    columns: BTreeMap<ComponentId, ArchetypeColumn>,
}

impl ArchetypeTable {
    pub(crate) fn new(
        component_columns: impl IntoIterator<Item = (ComponentId, TableColumnLayout)>,
    ) -> Self {
        let mut columns = BTreeMap::new();
        for (component_id, layout) in component_columns {
            columns
                .entry(component_id)
                .or_insert_with(|| ArchetypeColumn::new(layout));
        }
        Self {
            entities: Vec::new(),
            columns,
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

    pub(crate) fn component_ids(&self) -> impl ExactSizeIterator<Item = ComponentId> + '_ {
        self.columns.keys().copied()
    }

    pub(crate) fn append_row(
        &mut self,
        entity: EntityId,
        components: impl IntoIterator<Item = (ComponentId, StoredComponent, ComponentTicks)>,
    ) -> Result<usize, ArchetypeTableError> {
        let mut components = collect_row_components(components)?;
        self.validate_row_components(&components)?;

        let row = self.entities.len();
        for (component_id, column) in &mut self.columns {
            let (value, ticks) = components
                .remove(component_id)
                .expect("validated archetype table row must initialize every column");
            column.push(value, ticks);
        }
        debug_assert!(components.is_empty());
        self.entities.push(entity);
        Ok(row)
    }

    pub(crate) fn get<T>(&self, component_id: ComponentId, row: usize) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        self.columns.get(&component_id)?.get::<T>(row)
    }

    pub(crate) fn get_mut<T>(&mut self, component_id: ComponentId, row: usize) -> Option<&mut T>
    where
        T: Send + Sync + 'static,
    {
        self.columns.get_mut(&component_id)?.get_mut::<T>(row)
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
        self.columns
            .get_mut(&component_id)?
            .get_mut_at_tick::<T>(row, tick)
    }

    pub(crate) fn component_ticks(
        &self,
        component_id: ComponentId,
        row: usize,
    ) -> Option<ComponentTicks> {
        self.columns.get(&component_id)?.ticks(row)
    }

    /// Replaces one value in place, keeping the entity and every column row-aligned.
    pub(crate) fn replace(
        &mut self,
        component_id: ComponentId,
        row: usize,
        value: StoredComponent,
        tick: ChangeTick,
    ) -> Option<StoredComponent> {
        self.columns
            .get_mut(&component_id)?
            .replace(row, value, tick)
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

    fn validate_row_components(
        &self,
        components: &BTreeMap<ComponentId, (StoredComponent, ComponentTicks)>,
    ) -> Result<(), ArchetypeTableError> {
        for (component_id, (value, _)) in components {
            let Some(column) = self.columns.get(component_id) else {
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
        for component_id in self.columns.keys() {
            if !components.contains_key(component_id) {
                return Err(ArchetypeTableError::MissingComponentColumn {
                    component_id: *component_id,
                });
            }
        }
        Ok(())
    }
}

fn collect_row_components(
    components: impl IntoIterator<Item = (ComponentId, StoredComponent, ComponentTicks)>,
) -> Result<BTreeMap<ComponentId, (StoredComponent, ComponentTicks)>, ArchetypeTableError> {
    let mut values = BTreeMap::new();
    for (component_id, value, ticks) in components {
        match values.entry(component_id) {
            Entry::Vacant(entry) => {
                entry.insert((value, ticks));
            }
            Entry::Occupied(_) => {
                return Err(ArchetypeTableError::DuplicateComponentColumn { component_id });
            }
        }
    }
    Ok(values)
}
