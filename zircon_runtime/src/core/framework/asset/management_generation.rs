use std::collections::HashMap;
use std::sync::Arc;

use crate::core::resource::{ResourceId, ResourceKind, ResourceRecord, ResourceState};

pub(crate) const RESOURCE_MANAGEMENT_SHARD_COUNT: usize = 64;

/// Compact immutable resource row shared by runtime and authoring projections.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceManagementRow {
    pub id: ResourceId,
    pub kind: ResourceKind,
    pub primary_locator: Arc<str>,
    pub revision: u64,
    pub state: ResourceState,
    pub diagnostic_count: usize,
}

impl ResourceManagementRow {
    pub(crate) fn from_record(record: &ResourceRecord) -> Self {
        Self {
            id: record.id,
            kind: record.kind,
            primary_locator: Arc::from(record.primary_locator.to_string()),
            revision: record.revision,
            state: record.state,
            diagnostic_count: record.diagnostics.len(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResourceManagementKindSummary {
    pub total_count: usize,
    pub pending_count: usize,
    pub ready_count: usize,
    pub error_count: usize,
    pub reloading_count: usize,
}

impl ResourceManagementKindSummary {
    fn count_for_state(self, state: ResourceState) -> usize {
        match state {
            ResourceState::Pending => self.pending_count,
            ResourceState::Ready => self.ready_count,
            ResourceState::Error => self.error_count,
            ResourceState::Reloading => self.reloading_count,
        }
    }

    pub(crate) fn add(&mut self, state: ResourceState) {
        self.total_count += 1;
        match state {
            ResourceState::Pending => self.pending_count += 1,
            ResourceState::Ready => self.ready_count += 1,
            ResourceState::Error => self.error_count += 1,
            ResourceState::Reloading => self.reloading_count += 1,
        }
    }

    pub(crate) fn remove(&mut self, state: ResourceState) {
        self.total_count = self.total_count.saturating_sub(1);
        match state {
            ResourceState::Pending => self.pending_count = self.pending_count.saturating_sub(1),
            ResourceState::Ready => self.ready_count = self.ready_count.saturating_sub(1),
            ResourceState::Error => self.error_count = self.error_count.saturating_sub(1),
            ResourceState::Reloading => {
                self.reloading_count = self.reloading_count.saturating_sub(1)
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResourceManagementSummary {
    total_count: usize,
    by_kind: HashMap<ResourceKind, ResourceManagementKindSummary>,
}

impl ResourceManagementSummary {
    pub fn total_count(&self) -> usize {
        self.total_count
    }

    pub fn kind(&self, kind: ResourceKind) -> ResourceManagementKindSummary {
        self.by_kind.get(&kind).copied().unwrap_or_default()
    }

    fn matching_count(&self, query: ResourceManagementQuery) -> usize {
        match (query.kind, query.state) {
            (Some(kind), Some(state)) => self.kind(kind).count_for_state(state),
            (Some(kind), None) => self.kind(kind).total_count,
            (None, Some(state)) => self
                .by_kind
                .values()
                .map(|summary| summary.count_for_state(state))
                .sum(),
            (None, None) => self.total_count,
        }
    }

    pub(crate) fn add(&mut self, row: &ResourceManagementRow) {
        self.total_count += 1;
        self.by_kind.entry(row.kind).or_default().add(row.state);
    }

    pub(crate) fn remove(&mut self, row: &ResourceManagementRow) {
        self.total_count = self.total_count.saturating_sub(1);
        if let Some(summary) = self.by_kind.get_mut(&row.kind) {
            summary.remove(row.state);
            if summary.total_count == 0 {
                self.by_kind.remove(&row.kind);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResourceManagementQuery {
    pub kind: Option<ResourceKind>,
    pub state: Option<ResourceState>,
}

impl ResourceManagementQuery {
    fn matches(self, row: &ResourceManagementRow) -> bool {
        self.kind.is_none_or(|kind| row.kind == kind)
            && self.state.is_none_or(|state| row.state == state)
    }
}

#[derive(Clone, Debug)]
pub struct ResourceManagementPage {
    pub generation: u64,
    pub total_matching_count: usize,
    pub rows: Arc<[Arc<ResourceManagementRow>]>,
}

/// Incremental stable-order scan that never revisits a consumed shard prefix.
#[derive(Clone, Debug)]
pub struct ResourceManagementScan {
    generation: Arc<ResourceManagementGeneration>,
    query: ResourceManagementQuery,
    cursors: Vec<usize>,
    yielded_count: usize,
    total_matching_count: usize,
}

impl ResourceManagementScan {
    pub fn next_row(&mut self) -> Option<Arc<ResourceManagementRow>> {
        let mut selected: Option<(usize, Arc<ResourceManagementRow>)> = None;
        for (shard_index, shard) in self.generation.shards.iter().enumerate() {
            let rows = shard.rows();
            while self.cursors[shard_index] < rows.len()
                && !self.query.matches(&rows[self.cursors[shard_index]])
            {
                self.cursors[shard_index] += 1;
            }
            let Some(candidate) = rows.get(self.cursors[shard_index]) else {
                continue;
            };
            let replace = selected.as_ref().is_none_or(|(_, current)| {
                candidate.primary_locator < current.primary_locator
                    || (candidate.primary_locator == current.primary_locator
                        && candidate.id < current.id)
            });
            if replace {
                selected = Some((shard_index, candidate.clone()));
            }
        }
        let (shard_index, row) = selected?;
        self.cursors[shard_index] += 1;
        self.yielded_count = self.yielded_count.saturating_add(1);
        Some(row)
    }

    pub fn is_complete(&self) -> bool {
        self.yielded_count >= self.total_matching_count
    }

    pub fn total_matching_count(&self) -> usize {
        self.total_matching_count
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ResourceManagementShard {
    rows: Arc<[Arc<ResourceManagementRow>]>,
    by_id: HashMap<ResourceId, Arc<ResourceManagementRow>>,
}

impl ResourceManagementShard {
    pub(crate) fn from_rows(mut rows: Vec<Arc<ResourceManagementRow>>) -> Self {
        rows.sort_by(|left, right| {
            left.primary_locator
                .cmp(&right.primary_locator)
                .then_with(|| left.id.cmp(&right.id))
        });
        let by_id = rows.iter().map(|row| (row.id, row.clone())).collect();
        Self {
            rows: rows.into(),
            by_id,
        }
    }

    pub(crate) fn rows(&self) -> &[Arc<ResourceManagementRow>] {
        self.rows.as_ref()
    }

    fn row_by_id(&self, id: ResourceId) -> Option<Arc<ResourceManagementRow>> {
        self.by_id.get(&id).cloned()
    }

    fn row_by_locator(&self, locator: &str) -> Option<Arc<ResourceManagementRow>> {
        let index = self
            .rows
            .binary_search_by(|row| row.primary_locator.as_ref().cmp(locator))
            .ok()?;
        Some(self.rows[index].clone())
    }
}

/// Immutable, shard-backed resource projection published with the registry authority.
#[derive(Clone, Debug)]
pub struct ResourceManagementGeneration {
    sequence: u64,
    summary: ResourceManagementSummary,
    shards: Arc<[Arc<ResourceManagementShard>]>,
}

impl Default for ResourceManagementGeneration {
    fn default() -> Self {
        Self {
            sequence: 0,
            summary: ResourceManagementSummary::default(),
            shards: (0..RESOURCE_MANAGEMENT_SHARD_COUNT)
                .map(|_| Arc::new(ResourceManagementShard::default()))
                .collect::<Vec<_>>()
                .into(),
        }
    }
}

impl ResourceManagementGeneration {
    pub(crate) fn from_parts(
        sequence: u64,
        summary: ResourceManagementSummary,
        shards: Vec<Arc<ResourceManagementShard>>,
    ) -> Self {
        debug_assert_eq!(shards.len(), RESOURCE_MANAGEMENT_SHARD_COUNT);
        Self {
            sequence,
            summary,
            shards: shards.into(),
        }
    }

    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn summary(&self) -> &ResourceManagementSummary {
        &self.summary
    }

    pub fn row_by_id(&self, id: ResourceId) -> Option<Arc<ResourceManagementRow>> {
        self.shards[resource_management_shard_index(id)].row_by_id(id)
    }

    pub fn row_by_locator(&self, locator: &str) -> Option<Arc<ResourceManagementRow>> {
        self.shards
            .iter()
            .find_map(|shard| shard.row_by_locator(locator))
    }

    pub fn scan(self: &Arc<Self>, query: ResourceManagementQuery) -> ResourceManagementScan {
        ResourceManagementScan {
            generation: self.clone(),
            query,
            cursors: vec![0; self.shards.len()],
            yielded_count: 0,
            total_matching_count: self.summary.matching_count(query),
        }
    }

    pub fn page(
        &self,
        query: ResourceManagementQuery,
        offset: usize,
        limit: usize,
    ) -> ResourceManagementPage {
        let mut cursors = vec![0usize; self.shards.len()];
        let take_count = offset.saturating_add(limit);
        let mut ordered = Vec::with_capacity(take_count.min(self.summary.total_count));
        while ordered.len() < take_count {
            let mut selected: Option<(usize, Arc<ResourceManagementRow>)> = None;
            for (shard_index, shard) in self.shards.iter().enumerate() {
                let rows = shard.rows();
                while cursors[shard_index] < rows.len()
                    && !query.matches(&rows[cursors[shard_index]])
                {
                    cursors[shard_index] += 1;
                }
                let Some(candidate) = rows.get(cursors[shard_index]) else {
                    continue;
                };
                let replace = selected.as_ref().is_none_or(|(_, current)| {
                    candidate.primary_locator < current.primary_locator
                        || (candidate.primary_locator == current.primary_locator
                            && candidate.id < current.id)
                });
                if replace {
                    selected = Some((shard_index, candidate.clone()));
                }
            }
            let Some((shard_index, row)) = selected else {
                break;
            };
            cursors[shard_index] += 1;
            ordered.push(row);
        }
        let rows = ordered
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect::<Vec<_>>()
            .into();
        ResourceManagementPage {
            generation: self.sequence,
            total_matching_count: self.summary.matching_count(query),
            rows,
        }
    }

    pub(crate) fn shards(&self) -> &[Arc<ResourceManagementShard>] {
        self.shards.as_ref()
    }
}

pub(crate) fn resource_management_shard_index(id: ResourceId) -> usize {
    use std::hash::{DefaultHasher, Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    hasher.finish() as usize % RESOURCE_MANAGEMENT_SHARD_COUNT
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::resource::ResourceLocator;

    #[test]
    fn resource_management_scan_cursor_yields_matching_rows_once_in_stable_order() {
        let mut summary = ResourceManagementSummary::default();
        let mut rows_by_shard = vec![Vec::new(); RESOURCE_MANAGEMENT_SHARD_COUNT];
        for (label, locator, kind) in [
            ("scan-z", "res://scenes/z.scene.toml", ResourceKind::Scene),
            ("scan-a", "res://scenes/a.scene.toml", ResourceKind::Scene),
            ("scan-m", "res://meshes/m.mesh.toml", ResourceKind::Mesh),
        ] {
            let record = ResourceRecord::new(
                ResourceId::from_stable_label(label),
                kind,
                ResourceLocator::parse(locator).unwrap(),
            );
            let row = Arc::new(ResourceManagementRow::from_record(&record));
            summary.add(&row);
            rows_by_shard[resource_management_shard_index(row.id)].push(row);
        }
        let shards = rows_by_shard
            .into_iter()
            .map(|rows| Arc::new(ResourceManagementShard::from_rows(rows)))
            .collect();
        let generation = Arc::new(ResourceManagementGeneration::from_parts(7, summary, shards));
        let mut scan = generation.scan(ResourceManagementQuery {
            kind: Some(ResourceKind::Scene),
            state: None,
        });

        let first = scan.next_row().unwrap();
        let second = scan.next_row().unwrap();

        assert_eq!(first.primary_locator.as_ref(), "res://scenes/a.scene.toml");
        assert_eq!(second.primary_locator.as_ref(), "res://scenes/z.scene.toml");
        assert!(scan.next_row().is_none());
        assert!(scan.is_complete());
        assert_eq!(scan.total_matching_count(), 2);
    }
}
