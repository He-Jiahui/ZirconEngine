use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::scene::EntityId;
use crate::scene::ecs::{
    ChangeTick, ComponentId, ComponentTicks, component::TableColumnLayout, storage::StoredComponent,
};

use super::id::ArchetypeId;
use super::record::ArchetypeRecord;
use super::signature::ArchetypeSignature;
use super::table::{ArchetypePreflightedRow, ArchetypeTableError, ArchetypeTakenRow};

pub const ECS_ARCHETYPE_COMPONENT_INDEX_PROBES_DIAGNOSTIC: &str =
    "ecs.archetype.component_index_probes";
pub const ECS_ARCHETYPE_SIGNATURE_MEMBERSHIP_CHECKS_DIAGNOSTIC: &str =
    "ecs.archetype.signature_membership_checks";
pub const ECS_ARCHETYPE_ROW_APPENDS_DIAGNOSTIC: &str = "ecs.archetype.row_appends";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ArchetypeIndexPerformanceStats {
    pub component_index_probes: u64,
    pub signature_membership_checks: u64,
    pub row_appends: u64,
}

impl ArchetypeIndexPerformanceStats {
    pub(crate) fn saturating_delta_since(self, baseline: Self) -> Self {
        Self {
            component_index_probes: self
                .component_index_probes
                .saturating_sub(baseline.component_index_probes),
            signature_membership_checks: self
                .signature_membership_checks
                .saturating_sub(baseline.signature_membership_checks),
            row_appends: self.row_appends.saturating_sub(baseline.row_appends),
        }
    }

    pub(crate) fn diagnostic_values(&self) -> [(&'static str, f64); 3] {
        [
            (
                ECS_ARCHETYPE_COMPONENT_INDEX_PROBES_DIAGNOSTIC,
                self.component_index_probes as f64,
            ),
            (
                ECS_ARCHETYPE_SIGNATURE_MEMBERSHIP_CHECKS_DIAGNOSTIC,
                self.signature_membership_checks as f64,
            ),
            (
                ECS_ARCHETYPE_ROW_APPENDS_DIAGNOSTIC,
                self.row_appends as f64,
            ),
        ]
    }
}

#[derive(Debug, Default)]
struct ArchetypeIndexPerformanceCounters {
    component_index_probes: AtomicU64,
    signature_membership_checks: AtomicU64,
    row_appends: AtomicU64,
}

#[derive(Debug)]
pub struct ArchetypeIndex {
    records: Vec<ArchetypeRecord>,
    by_signature: HashMap<ArchetypeSignature, ArchetypeId>,
    by_component: HashMap<ComponentId, Vec<ArchetypeId>>,
    performance_counters: ArchetypeIndexPerformanceCounters,
}

#[derive(Clone, Copy, Debug)]
struct ArchetypeTopologySnapshot<'a> {
    index: &'a ArchetypeIndex,
}

impl PartialEq for ArchetypeTopologySnapshot<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.index.by_signature == other.index.by_signature
            && self.index.by_component == other.index.by_component
            && self.index.records.len() == other.index.records.len()
            && self
                .index
                .records
                .iter()
                .zip(&other.index.records)
                .all(|(left, right)| {
                    left.id() == right.id()
                        && left.signature() == right.signature()
                        && left.entities() == right.entities()
                })
    }
}

impl Eq for ArchetypeTopologySnapshot<'_> {}

impl ArchetypeIndex {
    pub fn new() -> Self {
        let empty = ArchetypeSignature::empty();
        let mut by_signature = HashMap::new();
        by_signature.insert(empty.clone(), ArchetypeId::EMPTY);
        Self {
            records: vec![ArchetypeRecord::new(ArchetypeId::EMPTY, empty, [])],
            by_signature,
            by_component: HashMap::new(),
            performance_counters: ArchetypeIndexPerformanceCounters::default(),
        }
    }

    pub fn generation(&self) -> u64 {
        self.records.len() as u64
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    fn topology_snapshot(&self) -> ArchetypeTopologySnapshot<'_> {
        ArchetypeTopologySnapshot { index: self }
    }

    pub(crate) fn estimated_heap_bytes(&self) -> usize {
        let record_bytes = self
            .records
            .capacity()
            .saturating_mul(std::mem::size_of::<ArchetypeRecord>());
        let record_heap_bytes = self.records.iter().fold(0_usize, |bytes, record| {
            bytes.saturating_add(record.estimated_heap_bytes())
        });
        let signature_index_bytes = self
            .by_signature
            .capacity()
            .saturating_mul(std::mem::size_of::<(ArchetypeSignature, ArchetypeId)>());
        let signature_key_bytes = self.by_signature.keys().fold(0_usize, |bytes, signature| {
            bytes.saturating_add(signature.estimated_heap_bytes())
        });
        let component_index_bytes = self
            .by_component
            .capacity()
            .saturating_mul(std::mem::size_of::<(ComponentId, Vec<ArchetypeId>)>());
        let component_rows_bytes = self
            .by_component
            .values()
            .fold(0_usize, |bytes, archetypes| {
                bytes.saturating_add(
                    archetypes
                        .capacity()
                        .saturating_mul(std::mem::size_of::<ArchetypeId>()),
                )
            });
        record_bytes
            .saturating_add(record_heap_bytes)
            .saturating_add(signature_index_bytes)
            .saturating_add(signature_key_bytes)
            .saturating_add(component_index_bytes)
            .saturating_add(component_rows_bytes)
    }

    pub(crate) fn performance_stats(&self) -> ArchetypeIndexPerformanceStats {
        ArchetypeIndexPerformanceStats {
            component_index_probes: self
                .performance_counters
                .component_index_probes
                .load(Ordering::Relaxed),
            signature_membership_checks: self
                .performance_counters
                .signature_membership_checks
                .load(Ordering::Relaxed),
            row_appends: self
                .performance_counters
                .row_appends
                .load(Ordering::Relaxed),
        }
    }

    fn record_component_index_probe(&self) {
        self.performance_counters
            .component_index_probes
            .fetch_add(1, Ordering::Relaxed);
    }

    fn record_signature_membership_check(&self) {
        self.performance_counters
            .signature_membership_checks
            .fetch_add(1, Ordering::Relaxed);
    }

    fn record_row_append(&self) {
        self.performance_counters
            .row_appends
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn signature(&self, id: ArchetypeId) -> Option<&ArchetypeSignature> {
        let record = self.records.get(id.index())?;
        Some(record.signature())
    }

    pub fn entities(&self, id: ArchetypeId) -> Option<&[EntityId]> {
        let record = self.records.get(id.index())?;
        Some(record.entities())
    }

    pub fn membership_generation(&self, id: ArchetypeId) -> Option<u64> {
        self.records
            .get(id.index())
            .map(ArchetypeRecord::membership_generation)
    }

    pub fn id_or_insert(
        &mut self,
        signature: ArchetypeSignature,
        table_columns: impl IntoIterator<Item = (ComponentId, TableColumnLayout)>,
    ) -> ArchetypeId {
        if let Some(id) = self.by_signature.get(&signature).copied() {
            return id;
        }

        let id = ArchetypeId::new(self.records.len());
        self.index_signature_components(id, &signature);
        self.records
            .push(ArchetypeRecord::new(id, signature.clone(), table_columns));
        self.by_signature.insert(signature, id);
        id
    }

    pub(crate) fn id_for_signature(&self, signature: &ArchetypeSignature) -> Option<ArchetypeId> {
        self.by_signature.get(signature).copied()
    }

    pub(crate) fn preflight_row(
        &self,
        id: ArchetypeId,
        components: impl IntoIterator<Item = (ComponentId, StoredComponent, ComponentTicks)>,
    ) -> Result<ArchetypePreflightedRow, ArchetypeTableError> {
        self.records
            .get(id.index())
            .expect("registered archetype id must own a record")
            .preflight_row(components)
    }

    pub(crate) fn validate_row_components(
        &self,
        id: ArchetypeId,
        components: &BTreeMap<ComponentId, (StoredComponent, ComponentTicks)>,
    ) -> Result<(), ArchetypeTableError> {
        self.records
            .get(id.index())
            .expect("registered archetype id must own a record")
            .validate_row_components(components)
    }

    pub(crate) fn validate_transition(
        &self,
        id: ArchetypeId,
        source_component_ids: impl IntoIterator<Item = ComponentId>,
        updates: &BTreeMap<ComponentId, Option<(StoredComponent, ComponentTicks)>>,
    ) -> Result<(), ArchetypeTableError> {
        self.records
            .get(id.index())
            .expect("registered archetype id must own a record")
            .validate_transition(source_component_ids, updates)
    }

    pub(crate) fn bind_prevalidated_row(
        &self,
        id: ArchetypeId,
        components: BTreeMap<ComponentId, (StoredComponent, ComponentTicks)>,
    ) -> ArchetypePreflightedRow {
        self.records
            .get(id.index())
            .expect("registered archetype id must own a record")
            .bind_prevalidated_row(components)
    }

    pub(crate) fn append_preflighted_row(
        &mut self,
        id: ArchetypeId,
        entity: EntityId,
        row: ArchetypePreflightedRow,
    ) -> usize {
        let row = self
            .records
            .get_mut(id.index())
            .expect("registered archetype id must own a record")
            .append_preflighted_row(entity, row);
        self.record_row_append();
        row
    }

    pub(crate) fn take_entity_row(
        &mut self,
        id: ArchetypeId,
        row: usize,
        entity: EntityId,
    ) -> Result<ArchetypeTakenRow, ArchetypeTableError> {
        self.records
            .get_mut(id.index())
            .ok_or(ArchetypeTableError::RowOutOfBounds { row, len: 0 })?
            .take_row(row, entity)
    }

    pub(crate) fn remove_entity_at(
        &mut self,
        id: ArchetypeId,
        row: usize,
        entity: EntityId,
    ) -> Option<(EntityId, usize)> {
        let taken = self.take_entity_row(id, row, entity).ok()?;
        taken.swapped_entity().map(|swapped| (swapped, row))
    }

    pub fn matching_archetypes(
        &self,
        required: &[ComponentId],
        without: &[ComponentId],
    ) -> Vec<ArchetypeId> {
        if let Some(ids) = self.shortest_required_archetype_ids(required) {
            if ids.is_empty() {
                return Vec::new();
            }

            let mut matches = Vec::with_capacity(ids.len());
            for id in ids {
                if self.archetype_matches_required_without(*id, required, without) {
                    matches.push(*id);
                }
            }
            return matches;
        }

        let mut matches = Vec::with_capacity(self.records.len());
        for record in &self.records {
            let id = record.id();
            if self.archetype_matches_required_without(id, required, without) {
                matches.push(id);
            }
        }
        matches
    }

    pub(crate) fn matching_archetypes_from(
        &self,
        required: &[ComponentId],
        without: &[ComponentId],
        first_archetype_index: usize,
    ) -> Vec<ArchetypeId> {
        (first_archetype_index..self.records.len())
            .map(ArchetypeId::new)
            .filter(|archetype| {
                self.archetype_matches_required_without(*archetype, required, without)
            })
            .collect()
    }

    fn shortest_required_archetype_ids(&self, required: &[ComponentId]) -> Option<&[ArchetypeId]> {
        let mut selected = None;
        let mut selected_len = usize::MAX;
        for component_id in required {
            self.record_component_index_probe();
            let ids = match self.by_component.get(component_id) {
                Some(ids) => ids.as_slice(),
                None => return Some(&[]),
            };
            let candidate_len = ids.len();
            if selected.is_none() || candidate_len < selected_len {
                selected = Some(ids);
                selected_len = candidate_len;
                if candidate_len == 0 {
                    break;
                }
            }
        }
        selected
    }

    fn archetype_matches_required_without(
        &self,
        id: ArchetypeId,
        required: &[ComponentId],
        without: &[ComponentId],
    ) -> bool {
        let Some(record) = self.records.get(id.index()) else {
            return false;
        };
        let signature = record.signature();
        for component_id in required {
            self.record_signature_membership_check();
            if !signature.contains(*component_id) {
                return false;
            }
        }
        for component_id in without {
            self.record_signature_membership_check();
            if signature.contains(*component_id) {
                return false;
            }
        }
        true
    }

    pub(crate) fn get<T>(
        &self,
        id: ArchetypeId,
        row: usize,
        component_id: ComponentId,
    ) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        self.records.get(id.index())?.get(component_id, row)
    }

    pub(crate) fn column_slot(&self, id: ArchetypeId, component_id: ComponentId) -> Option<usize> {
        self.records.get(id.index())?.column_slot(component_id)
    }

    pub(crate) fn get_by_slot<T>(
        &self,
        id: ArchetypeId,
        row: usize,
        column_slot: usize,
    ) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        self.records.get(id.index())?.get_by_slot(column_slot, row)
    }

    pub(crate) fn component_ticks_by_slot(
        &self,
        id: ArchetypeId,
        row: usize,
        column_slot: usize,
    ) -> Option<ComponentTicks> {
        self.records
            .get(id.index())?
            .component_ticks_by_slot(column_slot, row)
    }

    pub(crate) fn get_mut_at_tick_by_slot<T>(
        &mut self,
        id: ArchetypeId,
        row: usize,
        column_slot: usize,
        tick: ChangeTick,
    ) -> Option<&mut T>
    where
        T: Send + Sync + 'static,
    {
        self.records
            .get_mut(id.index())?
            .get_mut_at_tick_by_slot(column_slot, row, tick)
    }

    pub(crate) fn get_mut_with_ticks_by_slot<T>(
        &mut self,
        id: ArchetypeId,
        row: usize,
        column_slot: usize,
    ) -> Option<(&mut T, &mut ComponentTicks)>
    where
        T: Send + Sync + 'static,
    {
        self.records
            .get_mut(id.index())?
            .get_mut_with_ticks_by_slot(column_slot, row)
    }

    pub(crate) fn get_mut_at_tick<T>(
        &mut self,
        id: ArchetypeId,
        row: usize,
        component_id: ComponentId,
        tick: ChangeTick,
    ) -> Option<&mut T>
    where
        T: Send + Sync + 'static,
    {
        self.records
            .get_mut(id.index())?
            .get_mut_at_tick(component_id, row, tick)
    }

    pub(crate) fn get_mut_with_ticks<T>(
        &mut self,
        id: ArchetypeId,
        row: usize,
        component_id: ComponentId,
    ) -> Option<(&mut T, &mut ComponentTicks)>
    where
        T: Send + Sync + 'static,
    {
        self.records
            .get_mut(id.index())?
            .get_mut_with_ticks(component_id, row)
    }

    pub(crate) fn component_ticks(
        &self,
        id: ArchetypeId,
        row: usize,
        component_id: ComponentId,
    ) -> Option<ComponentTicks> {
        self.records
            .get(id.index())?
            .component_ticks(component_id, row)
    }

    pub(crate) fn replace(
        &mut self,
        id: ArchetypeId,
        row: usize,
        component_id: ComponentId,
        value: StoredComponent,
        tick: ChangeTick,
    ) -> Option<StoredComponent> {
        self.records
            .get_mut(id.index())?
            .replace(component_id, row, value, tick)
    }

    pub(crate) fn component_len(&self, component_id: ComponentId) -> usize {
        self.by_component
            .get(&component_id)
            .into_iter()
            .flatten()
            .filter_map(|id| self.records.get(id.index()))
            .map(|record| record.entities().len())
            .sum()
    }

    pub(crate) fn for_each_table_component<T>(
        &self,
        component_id: ComponentId,
        mut visit: impl FnMut(EntityId, &T),
    ) where
        T: Send + Sync + 'static,
    {
        let Some(archetypes) = self.by_component.get(&component_id) else {
            return;
        };
        for archetype in archetypes {
            self.records[archetype.index()].for_each_component(component_id, &mut visit);
        }
    }

    fn index_signature_components(&mut self, id: ArchetypeId, signature: &ArchetypeSignature) {
        for component_id in signature
            .table_components()
            .iter()
            .chain(signature.sparse_set_components())
            .copied()
        {
            let ids = self.by_component.entry(component_id).or_default();
            insert_archetype_id(ids, id);
        }
    }
}

fn insert_archetype_id(ids: &mut Vec<ArchetypeId>, id: ArchetypeId) {
    if let Err(index) = ids.binary_search(&id) {
        ids.insert(index, id);
    }
}

impl Default for ArchetypeIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for ArchetypeIndex {
    fn eq(&self, other: &Self) -> bool {
        self.topology_snapshot() == other.topology_snapshot()
    }
}

impl Eq for ArchetypeIndex {}

#[cfg(test)]
#[path = "index/tests.rs"]
mod tests;
