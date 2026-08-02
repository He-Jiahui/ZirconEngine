use std::any::TypeId;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

use crate::core::resource::readiness_generation::{
    resource_readiness_shard_index, ResourceReadinessGeneration,
    ResourceReadinessGenerationDiagnostics, ResourceReadinessRow, ResourceReadinessShard,
};
use crate::core::resource::{
    ResourceId, ResourceReadinessState, ResourceRecord, ResourceState, RuntimeResourceState,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResourceReadinessSource {
    record: Arc<ResourceRecord>,
    runtime_state: RuntimeResourceState,
    payload_type_id: Option<TypeId>,
}

#[derive(Clone, Debug)]
pub(super) struct ResourceReadinessSourceUpdate {
    pub(super) id: ResourceId,
    pub(super) record: Option<ResourceRecord>,
    pub(super) runtime_state: RuntimeResourceState,
    pub(super) payload_type_id: Option<TypeId>,
}

#[derive(Clone, Copy, Debug)]
struct ComputedAggregate {
    direct: ResourceReadinessState,
    recursive: ResourceReadinessState,
    fingerprint: u64,
}

#[derive(Debug, Default)]
pub(super) struct ResourceReadinessProjection {
    generation: Arc<ResourceReadinessGeneration>,
    sources: HashMap<ResourceId, ResourceReadinessSource>,
    reverse_dependencies: HashMap<ResourceId, HashSet<ResourceId>>,
}

impl ResourceReadinessProjection {
    pub(super) fn generation(&self) -> Arc<ResourceReadinessGeneration> {
        self.generation.clone()
    }

    pub(super) fn apply_updates(
        &mut self,
        updates: impl IntoIterator<Item = ResourceReadinessSourceUpdate>,
    ) {
        let mut changed_ids = HashSet::new();
        for update in updates {
            let next = update.record.map(|record| ResourceReadinessSource {
                record: Arc::new(record),
                runtime_state: update.runtime_state,
                payload_type_id: update.payload_type_id,
            });
            if self.sources.get(&update.id) == next.as_ref() {
                continue;
            }

            if let Some(previous) = self.sources.get(&update.id) {
                for dependency in &previous.record.dependency_ids {
                    if let Some(parents) = self.reverse_dependencies.get_mut(dependency) {
                        parents.remove(&update.id);
                    }
                }
            }
            if let Some(next) = &next {
                for dependency in &next.record.dependency_ids {
                    self.reverse_dependencies
                        .entry(*dependency)
                        .or_default()
                        .insert(update.id);
                }
                self.sources.insert(update.id, next.clone());
            } else {
                self.sources.remove(&update.id);
            }
            changed_ids.insert(update.id);
        }
        if changed_ids.is_empty() {
            return;
        }

        let affected = self.reverse_closure(changed_ids);
        self.publish_affected(affected);
    }

    fn reverse_closure(&self, roots: HashSet<ResourceId>) -> HashSet<ResourceId> {
        let mut affected = roots;
        let mut queue = affected.iter().copied().collect::<VecDeque<_>>();
        while let Some(id) = queue.pop_front() {
            if let Some(parents) = self.reverse_dependencies.get(&id) {
                for parent in parents {
                    if affected.insert(*parent) {
                        queue.push_back(*parent);
                    }
                }
            }
        }
        affected
    }

    fn publish_affected(&mut self, affected: HashSet<ResourceId>) {
        let mut ordered_affected = affected.iter().copied().collect::<Vec<_>>();
        ordered_affected.sort_unstable();
        let mut computed = HashMap::<ResourceId, ComputedAggregate>::new();
        let mut visiting = HashSet::new();
        let mut edge_visit_count = 0;
        for id in &ordered_affected {
            self.compute_aggregate(
                *id,
                &affected,
                &mut computed,
                &mut visiting,
                &mut edge_visit_count,
            );
        }

        let mut changed_shards =
            HashMap::<usize, HashMap<ResourceId, Arc<ResourceReadinessRow>>>::new();
        let mut changed_row_count = 0;
        for id in ordered_affected {
            let shard_index = resource_readiness_shard_index(id);
            let previous = self.generation.row(id).cloned();
            let next = self.sources.get(&id).map(|source| {
                let aggregate = computed
                    .get(&id)
                    .copied()
                    .expect("affected readiness aggregate");
                let load_state = source_load_state(source);
                let dependency_revision = match previous.as_ref() {
                    Some(previous) if previous.dependency_fingerprint == aggregate.fingerprint => {
                        previous.dependency_revision
                    }
                    Some(previous) => previous.dependency_revision.wrapping_add(1),
                    None => 1,
                };
                let candidate = ResourceReadinessRow {
                    record: source.record.clone(),
                    load_state,
                    direct_dependency_state: aggregate.direct,
                    recursive_dependency_state: if load_state == ResourceReadinessState::Loaded {
                        aggregate.recursive
                    } else {
                        load_state
                    },
                    dependency_revision,
                    dependency_fingerprint: aggregate.fingerprint,
                    payload_type_id: source.payload_type_id,
                };
                previous
                    .as_ref()
                    .filter(|previous| readiness_rows_equal(previous, &candidate))
                    .cloned()
                    .unwrap_or_else(|| Arc::new(candidate))
            });
            if same_optional_row(previous.as_ref(), next.as_ref()) {
                continue;
            }
            let shard = changed_shards
                .entry(shard_index)
                .or_insert_with(|| self.generation.shards()[shard_index].rows().clone());
            match next {
                Some(next) => {
                    shard.insert(id, next);
                }
                None => {
                    shard.remove(&id);
                }
            }
            changed_row_count += 1;
        }
        if changed_row_count == 0 {
            return;
        }

        let mut shards = self.generation.shards().to_vec();
        for (index, rows) in changed_shards {
            shards[index] = Arc::new(ResourceReadinessShard::from_rows(rows));
        }
        self.generation = Arc::new(ResourceReadinessGeneration::from_parts(
            self.generation.sequence().wrapping_add(1),
            ResourceReadinessGenerationDiagnostics {
                row_count: self.sources.len(),
                changed_row_count,
                affected_closure_count: affected.len(),
                edge_visit_count,
            },
            shards,
        ));
    }

    fn compute_aggregate(
        &self,
        id: ResourceId,
        affected: &HashSet<ResourceId>,
        computed: &mut HashMap<ResourceId, ComputedAggregate>,
        visiting: &mut HashSet<ResourceId>,
        edge_visit_count: &mut usize,
    ) -> ComputedAggregate {
        if let Some(computed) = computed.get(&id).copied() {
            return computed;
        }
        if !affected.contains(&id) {
            if let Some(previous) = self.generation.row(id) {
                return ComputedAggregate {
                    direct: previous.direct_dependency_state,
                    recursive: previous.recursive_dependency_state,
                    fingerprint: previous.dependency_fingerprint,
                };
            }
        }
        if !visiting.insert(id) {
            return ComputedAggregate {
                direct: ResourceReadinessState::Loaded,
                recursive: ResourceReadinessState::Loaded,
                fingerprint: hash_cycle_edge(id),
            };
        }

        let mut direct = None;
        let mut recursive = None;
        let mut fingerprint = DefaultHasher::new();
        if let Some(source) = self.sources.get(&id) {
            for dependency in &source.record.dependency_ids {
                *edge_visit_count = edge_visit_count.saturating_add(1);
                dependency.hash(&mut fingerprint);
                let dependency_state = self
                    .sources
                    .get(dependency)
                    .map(source_load_state)
                    .unwrap_or(ResourceReadinessState::Failed);
                dependency_state.hash(&mut fingerprint);
                direct = combine_state(direct, dependency_state);
                recursive = combine_state(recursive, dependency_state);
                if let Some(dependency_source) = self.sources.get(dependency) {
                    dependency_source.record.revision.hash(&mut fingerprint);
                    let nested = self.compute_aggregate(
                        *dependency,
                        affected,
                        computed,
                        visiting,
                        edge_visit_count,
                    );
                    nested.fingerprint.hash(&mut fingerprint);
                    recursive = combine_state(recursive, nested.recursive);
                }
            }
        }
        visiting.remove(&id);
        let result = ComputedAggregate {
            direct: direct.unwrap_or(ResourceReadinessState::Loaded),
            recursive: recursive.unwrap_or(ResourceReadinessState::Loaded),
            fingerprint: fingerprint.finish(),
        };
        computed.insert(id, result);
        result
    }
}

fn source_load_state(source: &ResourceReadinessSource) -> ResourceReadinessState {
    if source.record.state == ResourceState::Error
        || source.runtime_state == RuntimeResourceState::Error
    {
        return ResourceReadinessState::Failed;
    }
    if source.record.state == ResourceState::Reloading
        || source.runtime_state == RuntimeResourceState::Reloading
    {
        return ResourceReadinessState::Reloading;
    }
    if source.record.state == ResourceState::Pending
        || source.runtime_state == RuntimeResourceState::Loading
    {
        return ResourceReadinessState::Loading;
    }
    if source.record.state == ResourceState::Ready && source.payload_type_id.is_some() {
        return ResourceReadinessState::Loaded;
    }
    ResourceReadinessState::NotLoaded
}

fn combine_state(
    current: Option<ResourceReadinessState>,
    next: ResourceReadinessState,
) -> Option<ResourceReadinessState> {
    Some(match current {
        Some(current) if readiness_rank(current) >= readiness_rank(next) => current,
        _ => next,
    })
}

fn readiness_rank(state: ResourceReadinessState) -> u8 {
    match state {
        ResourceReadinessState::Loaded => 0,
        ResourceReadinessState::NotLoaded => 1,
        ResourceReadinessState::Loading => 2,
        ResourceReadinessState::Reloading => 3,
        ResourceReadinessState::Failed => 4,
    }
}

fn readiness_rows_equal(left: &ResourceReadinessRow, right: &ResourceReadinessRow) -> bool {
    left.record == right.record
        && left.load_state == right.load_state
        && left.direct_dependency_state == right.direct_dependency_state
        && left.recursive_dependency_state == right.recursive_dependency_state
        && left.dependency_revision == right.dependency_revision
        && left.dependency_fingerprint == right.dependency_fingerprint
        && left.payload_type_id == right.payload_type_id
}

fn same_optional_row(
    left: Option<&Arc<ResourceReadinessRow>>,
    right: Option<&Arc<ResourceReadinessRow>>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => Arc::ptr_eq(left, right),
        (None, None) => true,
        _ => false,
    }
}

fn hash_cycle_edge(id: ResourceId) -> u64 {
    let mut hasher = DefaultHasher::new();
    "resource-readiness-cycle".hash(&mut hasher);
    id.hash(&mut hasher);
    hasher.finish()
}
