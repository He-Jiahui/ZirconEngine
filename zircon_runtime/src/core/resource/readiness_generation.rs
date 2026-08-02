use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use super::{ResourceData, ResourceId, ResourceKind, ResourceRecord};

pub(crate) const RESOURCE_READINESS_SHARD_COUNT: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResourceReadinessState {
    NotLoaded,
    Loading,
    Loaded,
    Failed,
    Reloading,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResourceReadinessGenerationDiagnostics {
    pub row_count: usize,
    pub changed_row_count: usize,
    pub affected_closure_count: usize,
    pub edge_visit_count: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct ResourceReadinessRow {
    pub(crate) record: Arc<ResourceRecord>,
    pub(crate) load_state: ResourceReadinessState,
    pub(crate) direct_dependency_state: ResourceReadinessState,
    pub(crate) recursive_dependency_state: ResourceReadinessState,
    pub(crate) dependency_revision: u64,
    pub(crate) dependency_fingerprint: u64,
    pub(crate) payload_type_id: Option<TypeId>,
}

impl ResourceReadinessRow {
    pub(crate) fn typed_load_state<TData: ResourceData>(&self) -> ResourceReadinessState {
        if self.load_state == ResourceReadinessState::Loaded
            && self.payload_type_id != Some(TypeId::of::<TData>())
        {
            ResourceReadinessState::NotLoaded
        } else {
            self.load_state
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ResourceReadinessShard {
    rows: Arc<HashMap<ResourceId, Arc<ResourceReadinessRow>>>,
}

impl ResourceReadinessShard {
    pub(crate) fn from_rows(rows: HashMap<ResourceId, Arc<ResourceReadinessRow>>) -> Self {
        Self {
            rows: Arc::new(rows),
        }
    }

    pub(crate) fn rows(&self) -> &HashMap<ResourceId, Arc<ResourceReadinessRow>> {
        self.rows.as_ref()
    }

    fn row(&self, id: ResourceId) -> Option<&Arc<ResourceReadinessRow>> {
        self.rows.get(&id)
    }
}

#[derive(Clone, Debug)]
pub struct ResourceReadinessGeneration {
    sequence: u64,
    diagnostics: ResourceReadinessGenerationDiagnostics,
    shards: Arc<[Arc<ResourceReadinessShard>]>,
}

impl Default for ResourceReadinessGeneration {
    fn default() -> Self {
        Self {
            sequence: 0,
            diagnostics: ResourceReadinessGenerationDiagnostics::default(),
            shards: (0..RESOURCE_READINESS_SHARD_COUNT)
                .map(|_| Arc::new(ResourceReadinessShard::default()))
                .collect::<Vec<_>>()
                .into(),
        }
    }
}

impl ResourceReadinessGeneration {
    pub(crate) fn from_parts(
        sequence: u64,
        diagnostics: ResourceReadinessGenerationDiagnostics,
        shards: Vec<Arc<ResourceReadinessShard>>,
    ) -> Self {
        debug_assert_eq!(shards.len(), RESOURCE_READINESS_SHARD_COUNT);
        Self {
            sequence,
            diagnostics,
            shards: shards.into(),
        }
    }

    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn diagnostics(&self) -> ResourceReadinessGenerationDiagnostics {
        self.diagnostics
    }

    pub fn dependency_revision(&self, id: ResourceId) -> Option<u64> {
        self.row(id).map(|row| row.dependency_revision)
    }

    pub fn contains_kind(&self, id: ResourceId, kind: ResourceKind) -> bool {
        self.row(id).is_some_and(|row| row.record.kind == kind)
    }

    pub(crate) fn row(&self, id: ResourceId) -> Option<&Arc<ResourceReadinessRow>> {
        self.shards[resource_readiness_shard_index(id)].row(id)
    }

    pub(crate) fn shards(&self) -> &[Arc<ResourceReadinessShard>] {
        self.shards.as_ref()
    }
}

pub(crate) fn resource_readiness_shard_index(id: ResourceId) -> usize {
    use std::hash::{DefaultHasher, Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    hasher.finish() as usize % RESOURCE_READINESS_SHARD_COUNT
}
