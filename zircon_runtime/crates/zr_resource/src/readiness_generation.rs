use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
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
    pub publication_count: u64,
    pub row_count: usize,
    pub changed_row_count: usize,
    pub affected_closure_count: usize,
    pub edge_visit_count: usize,
}

#[derive(Clone, Debug)]
pub struct ResourceReadinessRow {
    pub record: Arc<ResourceRecord>,
    pub load_state: ResourceReadinessState,
    pub direct_dependency_state: ResourceReadinessState,
    pub recursive_dependency_state: ResourceReadinessState,
    pub(crate) dependency_revision: u64,
    pub(crate) dependency_fingerprint: u64,
    pub(crate) payload_type_id: Option<TypeId>,
}

impl ResourceReadinessRow {
    pub fn typed_load_state<TData: ResourceData>(&self) -> ResourceReadinessState {
        if self.load_state == ResourceReadinessState::Loaded
            && self.payload_type_id != Some(TypeId::of::<TData>())
        {
            ResourceReadinessState::NotLoaded
        } else {
            self.load_state
        }
    }
}

/// Process-local identity of one immutable readiness-row publication.
#[derive(Clone)]
pub struct ResourceReadinessRowIdentity(Arc<ResourceReadinessRow>);

impl ResourceReadinessRowIdentity {
    fn new(row: Arc<ResourceReadinessRow>) -> Self {
        Self(row)
    }

    pub fn row(&self) -> &ResourceReadinessRow {
        self.0.as_ref()
    }
}

impl fmt::Debug for ResourceReadinessRowIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("ResourceReadinessRowIdentity")
            .field(&self.0.record.id)
            .finish()
    }
}

impl PartialEq for ResourceReadinessRowIdentity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ResourceReadinessRowIdentity {}

impl Hash for ResourceReadinessRowIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(Arc::as_ptr(&self.0) as usize);
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
    diagnostics: ResourceReadinessGenerationDiagnostics,
    shards: Arc<[Arc<ResourceReadinessShard>]>,
}

/// Process-local identity of one immutable readiness-generation publication.
#[derive(Clone)]
pub struct ResourceReadinessGenerationIdentity(Arc<ResourceReadinessGeneration>);

impl ResourceReadinessGenerationIdentity {
    fn new(generation: Arc<ResourceReadinessGeneration>) -> Self {
        Self(generation)
    }

    pub fn generation(&self) -> &ResourceReadinessGeneration {
        self.0.as_ref()
    }
}

impl fmt::Debug for ResourceReadinessGenerationIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ResourceReadinessGenerationIdentity")
    }
}

impl PartialEq for ResourceReadinessGenerationIdentity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ResourceReadinessGenerationIdentity {}

impl Hash for ResourceReadinessGenerationIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(Arc::as_ptr(&self.0) as usize);
    }
}

impl Default for ResourceReadinessGeneration {
    fn default() -> Self {
        Self {
            diagnostics: ResourceReadinessGenerationDiagnostics::default(),
            shards: (0..RESOURCE_READINESS_SHARD_COUNT)
                .map(|_| Arc::new(ResourceReadinessShard::default()))
                .collect::<Vec<_>>()
                .into(),
        }
    }
}

impl ResourceReadinessGeneration {
    pub fn identity(self: &Arc<Self>) -> ResourceReadinessGenerationIdentity {
        ResourceReadinessGenerationIdentity::new(Arc::clone(self))
    }

    pub(crate) fn from_parts(
        diagnostics: ResourceReadinessGenerationDiagnostics,
        shards: Vec<Arc<ResourceReadinessShard>>,
    ) -> Self {
        debug_assert_eq!(shards.len(), RESOURCE_READINESS_SHARD_COUNT);
        Self {
            diagnostics,
            shards: shards.into(),
        }
    }

    pub fn diagnostics(&self) -> ResourceReadinessGenerationDiagnostics {
        self.diagnostics
    }

    pub fn row_identity(&self, id: ResourceId) -> Option<ResourceReadinessRowIdentity> {
        self.row(id).cloned().map(ResourceReadinessRowIdentity::new)
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
    use std::hash::DefaultHasher;

    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    hasher.finish() as usize % RESOURCE_READINESS_SHARD_COUNT
}
