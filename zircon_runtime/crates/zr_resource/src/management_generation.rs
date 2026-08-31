use std::cmp::Ordering;
use std::collections::{HashMap, hash_map::RandomState};
use std::fmt;
use std::hash::{BuildHasher, Hash, Hasher};
use std::sync::Arc;

use crate::{ResourceId, ResourceKind, ResourceRecord, ResourceState};

pub(crate) const RESOURCE_MANAGEMENT_ID_SHARD_COUNT: usize = 1_024;
pub(crate) const RESOURCE_MANAGEMENT_LOCATOR_SHARD_COUNT: usize = 1_024;
pub(crate) const RESOURCE_MANAGEMENT_ORDERED_PAGE_ROWS: usize = 256;

#[derive(Clone, Debug)]
pub(crate) struct ResourceManagementHashAuthority {
    id_shard_hasher: RandomState,
    id_map_hasher: RandomState,
    locator_shard_hasher: RandomState,
}

impl Default for ResourceManagementHashAuthority {
    fn default() -> Self {
        Self {
            id_shard_hasher: RandomState::new(),
            id_map_hasher: RandomState::new(),
            locator_shard_hasher: RandomState::new(),
        }
    }
}

impl ResourceManagementHashAuthority {
    fn id_shard_index(&self, id: ResourceId) -> usize {
        debug_assert!(RESOURCE_MANAGEMENT_ID_SHARD_COUNT.is_power_of_two());
        self.id_shard_hasher.hash_one(id) as usize & (RESOURCE_MANAGEMENT_ID_SHARD_COUNT - 1)
    }

    fn id_map_with_capacity<V>(&self, capacity: usize) -> ResourceManagementIdMap<V> {
        ResourceManagementIdMap::with_capacity_and_hasher(capacity, self.id_map_hasher.clone())
    }

    fn locator_shard_index(&self, locator: &str) -> usize {
        debug_assert!(RESOURCE_MANAGEMENT_LOCATOR_SHARD_COUNT.is_power_of_two());
        self.locator_shard_hasher.hash_one(locator) as usize
            & (RESOURCE_MANAGEMENT_LOCATOR_SHARD_COUNT - 1)
    }
}

pub(crate) type ResourceManagementIdMap<V> = HashMap<ResourceId, V, RandomState>;

/// Observation-only counters for one management publication.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResourceManagementGenerationDiagnostics {
    pub publication_count: u64,
}

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
    #[cfg(test)]
    pub(crate) fn from_record(record: &ResourceRecord) -> Self {
        Self::from_record_reusing_identity(record, None)
    }

    pub(crate) fn from_record_reusing_identity(
        record: &ResourceRecord,
        previous: Option<&Self>,
    ) -> Self {
        let primary_locator = previous
            .filter(|row| {
                record
                    .primary_locator
                    .matches_display(row.primary_locator.as_ref())
            })
            .map(|row| Arc::clone(&row.primary_locator))
            .unwrap_or_else(|| Arc::from(record.primary_locator.to_string()));
        Self {
            id: record.id,
            kind: record.kind,
            primary_locator,
            revision: record.revision,
            state: record.state,
            diagnostic_count: record.diagnostics.len(),
        }
    }
}

/// Process-local identity of one immutable management row publication.
#[derive(Clone)]
pub struct ResourceManagementRowIdentity(Arc<ResourceManagementRow>);

impl ResourceManagementRowIdentity {
    fn new(row: Arc<ResourceManagementRow>) -> Self {
        Self(row)
    }

    pub fn row(&self) -> &ResourceManagementRow {
        self.0.as_ref()
    }
}

impl fmt::Debug for ResourceManagementRowIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("ResourceManagementRowIdentity")
            .field(&self.0.id)
            .finish()
    }
}

impl PartialEq for ResourceManagementRowIdentity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ResourceManagementRowIdentity {}

impl Hash for ResourceManagementRowIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(Arc::as_ptr(&self.0) as usize);
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

#[derive(Clone)]
pub struct ResourceManagementPage {
    pub generation: ResourceManagementGenerationIdentity,
    pub total_matching_count: usize,
    pub rows: Arc<[Arc<ResourceManagementRow>]>,
}

impl fmt::Debug for ResourceManagementPage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResourceManagementPage")
            .field("generation", &self.generation)
            .field("total_matching_count", &self.total_matching_count)
            .field("rows", &self.rows)
            .finish()
    }
}

/// Query-local work counters consumed by crate-owned profiling tests and benchmarks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ResourceManagementPageProfileMetrics {
    pub(crate) shard_candidate_checks: u64,
    pub(crate) filtered_rows_skipped: u64,
    pub(crate) candidate_rows: u64,
    pub(crate) rows_returned: u64,
}

/// Incremental stable-order scan over immutable globally ordered pages.
#[derive(Clone, Debug)]
pub struct ResourceManagementScan {
    generation: Arc<ResourceManagementGeneration>,
    query: ResourceManagementQuery,
    page_index: usize,
    row_index: usize,
    yielded_count: usize,
    total_matching_count: usize,
    #[cfg(feature = "profiling")]
    profile_metrics: ResourceManagementScanProfileMetrics,
}

/// Query-local work counters read after a profiling scan completes.
#[cfg(feature = "profiling")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ResourceManagementScanProfileMetrics {
    pub(crate) shard_candidate_checks: u64,
    pub(crate) filtered_rows_skipped: u64,
    pub(crate) rows_emitted: u64,
}

impl ResourceManagementScan {
    pub fn next_row(&mut self) -> Option<Arc<ResourceManagementRow>> {
        while let Some(page) = self.generation.ordered_pages.get(self.page_index) {
            if self.row_index == 0 {
                #[cfg(feature = "profiling")]
                {
                    self.profile_metrics.shard_candidate_checks = self
                        .profile_metrics
                        .shard_candidate_checks
                        .saturating_add(1);
                }
            }
            while let Some(row) = page.get(self.row_index) {
                self.row_index += 1;
                if !self.query.matches(row) {
                    #[cfg(feature = "profiling")]
                    {
                        self.profile_metrics.filtered_rows_skipped =
                            self.profile_metrics.filtered_rows_skipped.saturating_add(1);
                    }
                    continue;
                }
                self.yielded_count = self.yielded_count.saturating_add(1);
                #[cfg(feature = "profiling")]
                {
                    self.profile_metrics.rows_emitted =
                        self.profile_metrics.rows_emitted.saturating_add(1);
                }
                return Some(Arc::clone(row));
            }
            self.page_index += 1;
            self.row_index = 0;
        }
        None
    }

    pub fn is_complete(&self) -> bool {
        self.yielded_count >= self.total_matching_count
    }

    pub fn total_matching_count(&self) -> usize {
        self.total_matching_count
    }

    #[cfg(feature = "profiling")]
    pub(crate) fn profile_metrics(&self) -> ResourceManagementScanProfileMetrics {
        self.profile_metrics
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ResourceManagementIdShard {
    by_id: ResourceManagementIdMap<Arc<ResourceManagementRow>>,
}

impl ResourceManagementIdShard {
    pub(crate) fn from_entries(by_id: ResourceManagementIdMap<Arc<ResourceManagementRow>>) -> Self {
        Self { by_id }
    }

    pub(crate) fn entries(&self) -> &ResourceManagementIdMap<Arc<ResourceManagementRow>> {
        &self.by_id
    }

    fn row_by_id(&self, id: ResourceId) -> Option<Arc<ResourceManagementRow>> {
        self.by_id.get(&id).cloned()
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ResourceManagementLocatorShard {
    by_locator: HashMap<Arc<str>, ResourceId>,
}

impl ResourceManagementLocatorShard {
    pub(crate) fn from_entries(by_locator: HashMap<Arc<str>, ResourceId>) -> Self {
        Self { by_locator }
    }

    pub(crate) fn entries(&self) -> &HashMap<Arc<str>, ResourceId> {
        &self.by_locator
    }

    fn id_by_locator(&self, locator: &str) -> Option<ResourceId> {
        self.by_locator.get(locator).copied()
    }
}

/// Immutable resource projection with independent lookup and canonical-order storage.
#[derive(Clone, Debug)]
pub struct ResourceManagementGeneration {
    diagnostics: ResourceManagementGenerationDiagnostics,
    summary: ResourceManagementSummary,
    hash_authority: Arc<ResourceManagementHashAuthority>,
    ordered_pages: Arc<[Arc<[Arc<ResourceManagementRow>]>]>,
    id_shards: Arc<[Arc<ResourceManagementIdShard>]>,
    locator_shards: Arc<[Arc<ResourceManagementLocatorShard>]>,
}

/// Process-local identity of one immutable management generation publication.
#[derive(Clone)]
pub struct ResourceManagementGenerationIdentity(Arc<ResourceManagementGeneration>);

impl ResourceManagementGenerationIdentity {
    fn new(generation: Arc<ResourceManagementGeneration>) -> Self {
        Self(generation)
    }

    pub fn generation(&self) -> &ResourceManagementGeneration {
        self.0.as_ref()
    }
}

impl fmt::Debug for ResourceManagementGenerationIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ResourceManagementGenerationIdentity")
    }
}

impl PartialEq for ResourceManagementGenerationIdentity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ResourceManagementGenerationIdentity {}

impl Hash for ResourceManagementGenerationIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(Arc::as_ptr(&self.0) as usize);
    }
}

impl Default for ResourceManagementGeneration {
    fn default() -> Self {
        let hash_authority = Arc::new(ResourceManagementHashAuthority::default());
        let empty_id_shard = Arc::new(ResourceManagementIdShard::from_entries(
            hash_authority.id_map_with_capacity(0),
        ));
        let empty_locator_shard = Arc::new(ResourceManagementLocatorShard::default());
        Self {
            diagnostics: ResourceManagementGenerationDiagnostics::default(),
            summary: ResourceManagementSummary::default(),
            hash_authority,
            ordered_pages: Arc::from([]),
            id_shards: vec![empty_id_shard; RESOURCE_MANAGEMENT_ID_SHARD_COUNT].into(),
            locator_shards: vec![empty_locator_shard; RESOURCE_MANAGEMENT_LOCATOR_SHARD_COUNT]
                .into(),
        }
    }
}

impl ResourceManagementGeneration {
    pub fn identity(self: &Arc<Self>) -> ResourceManagementGenerationIdentity {
        ResourceManagementGenerationIdentity::new(Arc::clone(self))
    }

    pub(crate) fn from_parts(
        diagnostics: ResourceManagementGenerationDiagnostics,
        summary: ResourceManagementSummary,
        hash_authority: Arc<ResourceManagementHashAuthority>,
        ordered_pages: Arc<[Arc<[Arc<ResourceManagementRow>]>]>,
        id_shards: Arc<[Arc<ResourceManagementIdShard>]>,
        locator_shards: Arc<[Arc<ResourceManagementLocatorShard>]>,
    ) -> Self {
        debug_assert_eq!(id_shards.len(), RESOURCE_MANAGEMENT_ID_SHARD_COUNT);
        debug_assert_eq!(
            locator_shards.len(),
            RESOURCE_MANAGEMENT_LOCATOR_SHARD_COUNT
        );
        debug_assert!(ordered_pages.iter().all(|page| !page.is_empty()));
        debug_assert!(
            ordered_pages
                .iter()
                .flat_map(|page| page.iter())
                .is_sorted_by(|left, right| { resource_management_row_order(left, right).is_le() })
        );
        Self {
            diagnostics,
            summary,
            hash_authority,
            ordered_pages,
            id_shards,
            locator_shards,
        }
    }

    #[cfg(test)]
    pub(crate) fn from_rows(
        publication_count: u64,
        mut rows: Vec<Arc<ResourceManagementRow>>,
    ) -> Self {
        rows.sort_by(|left, right| resource_management_row_order(left, right));
        Self::from_sorted_rows(publication_count, rows)
    }

    #[cfg(test)]
    pub(crate) fn from_sorted_rows(
        publication_count: u64,
        rows: Vec<Arc<ResourceManagementRow>>,
    ) -> Self {
        Self::from_sorted_rows_with_hash_authority(
            ResourceManagementGenerationDiagnostics { publication_count },
            rows,
            Arc::new(ResourceManagementHashAuthority::default()),
        )
    }

    pub(crate) fn from_sorted_rows_with_hash_authority(
        diagnostics: ResourceManagementGenerationDiagnostics,
        rows: Vec<Arc<ResourceManagementRow>>,
        hash_authority: Arc<ResourceManagementHashAuthority>,
    ) -> Self {
        debug_assert!(
            rows.is_sorted_by(|left, right| { resource_management_row_order(left, right).is_le() })
        );
        let mut summary = ResourceManagementSummary::default();
        let mut locator_entries = (0..RESOURCE_MANAGEMENT_LOCATOR_SHARD_COUNT)
            .map(|_| HashMap::new())
            .collect::<Vec<_>>();
        for row in &rows {
            summary.add(row);
            locator_entries[hash_authority.locator_shard_index(&row.primary_locator)]
                .insert(Arc::clone(&row.primary_locator), row.id);
        }
        let ordered_pages = resource_management_pages_from_sorted_rows(rows);
        let id_entries =
            resource_management_id_maps_from_ordered_pages(&ordered_pages, hash_authority.as_ref());
        Self::from_parts(
            diagnostics,
            summary,
            hash_authority,
            ordered_pages,
            id_entries
                .into_iter()
                .map(ResourceManagementIdShard::from_entries)
                .map(Arc::new)
                .collect::<Vec<_>>()
                .into(),
            locator_entries
                .into_iter()
                .map(ResourceManagementLocatorShard::from_entries)
                .map(Arc::new)
                .collect::<Vec<_>>()
                .into(),
        )
    }

    pub fn diagnostics(&self) -> ResourceManagementGenerationDiagnostics {
        self.diagnostics
    }

    pub fn summary(&self) -> &ResourceManagementSummary {
        &self.summary
    }

    pub fn row_by_id(&self, id: ResourceId) -> Option<Arc<ResourceManagementRow>> {
        self.id_shards[self.id_shard_index(id)].row_by_id(id)
    }

    pub fn row_identity_by_id(&self, id: ResourceId) -> Option<ResourceManagementRowIdentity> {
        self.row_by_id(id).map(ResourceManagementRowIdentity::new)
    }

    pub fn row_by_locator(&self, locator: &str) -> Option<Arc<ResourceManagementRow>> {
        let shard = &self.locator_shards[self.locator_shard_index(locator)];
        self.row_by_id(shard.id_by_locator(locator)?)
    }

    pub fn row_identity_by_locator(&self, locator: &str) -> Option<ResourceManagementRowIdentity> {
        self.row_by_locator(locator)
            .map(ResourceManagementRowIdentity::new)
    }

    pub fn scan(self: &Arc<Self>, query: ResourceManagementQuery) -> ResourceManagementScan {
        ResourceManagementScan {
            generation: Arc::clone(self),
            query,
            page_index: 0,
            row_index: 0,
            yielded_count: 0,
            total_matching_count: self.summary.matching_count(query),
            #[cfg(feature = "profiling")]
            profile_metrics: ResourceManagementScanProfileMetrics::default(),
        }
    }

    pub fn page(
        self: &Arc<Self>,
        query: ResourceManagementQuery,
        offset: usize,
        limit: usize,
    ) -> ResourceManagementPage {
        self.page_with_profile_metrics::<false>(query, offset, limit)
            .0
    }

    #[cfg(feature = "profiling")]
    pub(crate) fn profiled_page(
        self: &Arc<Self>,
        query: ResourceManagementQuery,
        offset: usize,
        limit: usize,
    ) -> (ResourceManagementPage, ResourceManagementPageProfileMetrics) {
        self.page_with_profile_metrics::<true>(query, offset, limit)
    }

    fn page_with_profile_metrics<const PROFILE: bool>(
        self: &Arc<Self>,
        query: ResourceManagementQuery,
        offset: usize,
        limit: usize,
    ) -> (ResourceManagementPage, ResourceManagementPageProfileMetrics) {
        let total_matching_count = self.summary.matching_count(query);
        let mut metrics = ResourceManagementPageProfileMetrics::default();
        let mut matching_index = 0usize;
        let mut rows = Vec::with_capacity(limit.min(total_matching_count.saturating_sub(offset)));
        if limit != 0 {
            'pages: for page in self.ordered_pages.iter() {
                if PROFILE {
                    metrics.shard_candidate_checks =
                        metrics.shard_candidate_checks.saturating_add(1);
                }
                for row in page.iter() {
                    if !query.matches(row) {
                        if PROFILE {
                            metrics.filtered_rows_skipped =
                                metrics.filtered_rows_skipped.saturating_add(1);
                        }
                        continue;
                    }
                    if PROFILE {
                        metrics.candidate_rows = metrics.candidate_rows.saturating_add(1);
                    }
                    if matching_index >= offset {
                        rows.push(Arc::clone(row));
                        if rows.len() == limit {
                            break 'pages;
                        }
                    }
                    matching_index = matching_index.saturating_add(1);
                }
            }
        }
        if PROFILE {
            metrics.rows_returned = rows.len() as u64;
        }
        (
            ResourceManagementPage {
                generation: self.identity(),
                total_matching_count,
                rows: rows.into(),
            },
            metrics,
        )
    }

    pub(crate) fn ordered_pages(&self) -> &[Arc<[Arc<ResourceManagementRow>]>] {
        &self.ordered_pages
    }

    pub(crate) fn ordered_rows(&self) -> impl Iterator<Item = &Arc<ResourceManagementRow>> {
        self.ordered_pages.iter().flat_map(|page| page.iter())
    }

    pub(crate) fn id_shards(&self) -> &[Arc<ResourceManagementIdShard>] {
        &self.id_shards
    }

    pub(crate) fn id_shards_arc(&self) -> Arc<[Arc<ResourceManagementIdShard>]> {
        Arc::clone(&self.id_shards)
    }

    pub(crate) fn hash_authority_arc(&self) -> Arc<ResourceManagementHashAuthority> {
        Arc::clone(&self.hash_authority)
    }

    pub(crate) fn id_shard_index(&self, id: ResourceId) -> usize {
        self.hash_authority.id_shard_index(id)
    }

    pub(crate) fn locator_shard_index(&self, locator: &str) -> usize {
        self.hash_authority.locator_shard_index(locator)
    }

    pub(crate) fn locator_shards(&self) -> &[Arc<ResourceManagementLocatorShard>] {
        &self.locator_shards
    }

    pub(crate) fn locator_shards_arc(&self) -> Arc<[Arc<ResourceManagementLocatorShard>]> {
        Arc::clone(&self.locator_shards)
    }
}

pub(crate) fn resource_management_row_order(
    left: &ResourceManagementRow,
    right: &ResourceManagementRow,
) -> Ordering {
    left.primary_locator
        .cmp(&right.primary_locator)
        .then_with(|| left.id.cmp(&right.id))
}

pub(crate) fn resource_management_pages_from_sorted_rows(
    rows: Vec<Arc<ResourceManagementRow>>,
) -> Arc<[Arc<[Arc<ResourceManagementRow>]>]> {
    if rows.is_empty() {
        return Arc::from([]);
    }
    let page_count = rows.len().div_ceil(RESOURCE_MANAGEMENT_ORDERED_PAGE_ROWS);
    let rows_per_page = rows.len() / page_count;
    let larger_page_count = rows.len() % page_count;
    let mut rows = rows.into_iter();
    (0..page_count)
        .map(|page_index| {
            let page_len = rows_per_page + usize::from(page_index < larger_page_count);
            rows.by_ref().take(page_len).collect::<Vec<_>>().into()
        })
        .collect::<Vec<_>>()
        .into()
}

pub(crate) fn resource_management_id_maps_from_ordered_pages(
    ordered_pages: &[Arc<[Arc<ResourceManagementRow>]>],
    hash_authority: &ResourceManagementHashAuthority,
) -> Vec<ResourceManagementIdMap<Arc<ResourceManagementRow>>> {
    let mut counts = vec![0usize; RESOURCE_MANAGEMENT_ID_SHARD_COUNT];
    for row in ordered_pages.iter().flat_map(|page| page.iter()) {
        let index = hash_authority.id_shard_index(row.id);
        counts[index] = counts[index].saturating_add(1);
    }
    let mut entries = counts
        .into_iter()
        .map(|capacity| hash_authority.id_map_with_capacity(capacity))
        .collect::<Vec<_>>();
    for row in ordered_pages.iter().flat_map(|page| page.iter()) {
        entries[hash_authority.id_shard_index(row.id)].insert(row.id, Arc::clone(row));
    }
    entries
}

#[cfg(test)]
mod tests;
