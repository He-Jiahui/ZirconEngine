use std::any::TypeId;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

use crate::readiness_generation::{
    resource_readiness_shard_index, ResourceReadinessGeneration,
    ResourceReadinessGenerationDiagnostics, ResourceReadinessRow, ResourceReadinessShard,
};
use crate::{
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

#[derive(Clone, Copy, Debug)]
struct TraversalFrame {
    id: ResourceId,
    next_dependency: usize,
    parent: Option<ResourceId>,
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
            if source_matches_update(self.sources.get(&update.id), &update) {
                continue;
            }
            let next = update.record.map(|mut record| {
                record.dependency_ids.sort_unstable();
                record.dependency_ids.dedup();
                ResourceReadinessSource {
                    record: Arc::new(record),
                    runtime_state: update.runtime_state,
                    payload_type_id: update.payload_type_id,
                }
            });

            if let Some(previous) = self.sources.get(&update.id) {
                for dependency in &previous.record.dependency_ids {
                    let remove_bucket = match self.reverse_dependencies.get_mut(dependency) {
                        Some(parents) => {
                            parents.remove(&update.id);
                            parents.is_empty()
                        }
                        None => false,
                    };
                    if remove_bucket {
                        self.reverse_dependencies.remove(dependency);
                    }
                }
            }
            if let Some(next) = next {
                for dependency in &next.record.dependency_ids {
                    self.reverse_dependencies
                        .entry(*dependency)
                        .or_default()
                        .insert(update.id);
                }
                self.sources.insert(update.id, next);
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
        let (computed, edge_visit_count) = self.compute_aggregates(&ordered_affected, &affected);

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
                    Some(previous) => previous.dependency_revision.saturating_add(1),
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
            ResourceReadinessGenerationDiagnostics {
                publication_count: self
                    .generation
                    .diagnostics()
                    .publication_count
                    .saturating_add(1),
                row_count: self.sources.len(),
                changed_row_count,
                affected_closure_count: affected.len(),
                edge_visit_count,
            },
            shards,
        ));
    }

    fn compute_aggregates(
        &self,
        ordered_affected: &[ResourceId],
        affected: &HashSet<ResourceId>,
    ) -> (HashMap<ResourceId, ComputedAggregate>, usize) {
        let mut index_by_id = HashMap::<ResourceId, usize>::with_capacity(affected.len());
        let mut lowlink_by_id = HashMap::<ResourceId, usize>::with_capacity(affected.len());
        let mut component_stack = Vec::<ResourceId>::with_capacity(affected.len());
        let mut on_component_stack = HashSet::<ResourceId>::with_capacity(affected.len());
        let mut components = Vec::<Vec<ResourceId>>::new();
        let mut next_index = 0usize;
        let mut edge_visit_count = 0usize;

        for root in ordered_affected {
            if index_by_id.contains_key(root) {
                continue;
            }
            index_by_id.insert(*root, next_index);
            lowlink_by_id.insert(*root, next_index);
            next_index += 1;
            component_stack.push(*root);
            on_component_stack.insert(*root);
            let mut traversal = vec![TraversalFrame {
                id: *root,
                next_dependency: 0,
                parent: None,
            }];

            while !traversal.is_empty() {
                let frame_index = traversal.len() - 1;
                let frame_id = traversal[frame_index].id;
                let dependencies = self
                    .sources
                    .get(&frame_id)
                    .map(|source| source.record.dependency_ids.as_slice())
                    .unwrap_or(&[]);
                let next = dependencies
                    .get(traversal[frame_index].next_dependency)
                    .copied();
                traversal[frame_index].next_dependency += usize::from(next.is_some());
                if let Some(dependency) = next {
                    edge_visit_count = edge_visit_count.saturating_add(1);
                    if !affected.contains(&dependency) || !self.sources.contains_key(&dependency) {
                        continue;
                    }
                    if !index_by_id.contains_key(&dependency) {
                        index_by_id.insert(dependency, next_index);
                        lowlink_by_id.insert(dependency, next_index);
                        next_index += 1;
                        component_stack.push(dependency);
                        on_component_stack.insert(dependency);
                        traversal.push(TraversalFrame {
                            id: dependency,
                            next_dependency: 0,
                            parent: Some(frame_id),
                        });
                    } else if on_component_stack.contains(&dependency) {
                        let dependency_index = index_by_id[&dependency];
                        let lowlink = lowlink_by_id
                            .get_mut(&frame_id)
                            .expect("active readiness traversal lowlink");
                        *lowlink = (*lowlink).min(dependency_index);
                    }
                    continue;
                }

                let completed = traversal.pop().expect("active readiness traversal frame");
                let completed_lowlink = lowlink_by_id[&completed.id];
                if let Some(parent) = completed.parent {
                    let parent_lowlink = lowlink_by_id
                        .get_mut(&parent)
                        .expect("readiness traversal parent lowlink");
                    *parent_lowlink = (*parent_lowlink).min(completed_lowlink);
                }
                if completed_lowlink == index_by_id[&completed.id] {
                    let mut component = Vec::new();
                    loop {
                        let member = component_stack
                            .pop()
                            .expect("readiness component stack member");
                        on_component_stack.remove(&member);
                        component.push(member);
                        if member == completed.id {
                            break;
                        }
                    }
                    component.sort_unstable();
                    components.push(component);
                }
            }
        }

        let mut component_by_id = HashMap::with_capacity(affected.len());
        for (component_index, component) in components.iter().enumerate() {
            for member in component {
                component_by_id.insert(*member, component_index);
            }
        }
        let cyclic = components
            .iter()
            .map(|component| {
                component.len() > 1
                    || component.first().is_some_and(|member| {
                        self.sources.get(member).is_some_and(|source| {
                            source.record.dependency_ids.binary_search(member).is_ok()
                        })
                    })
            })
            .collect::<Vec<_>>();
        let cycle_fingerprints = components
            .iter()
            .enumerate()
            .map(|(component_index, component)| {
                cyclic[component_index].then(|| {
                    let mut fingerprint = DefaultHasher::new();
                    hash_cycle_component(component, &mut fingerprint);
                    fingerprint.finish()
                })
            })
            .collect::<Vec<_>>();

        let mut computed = HashMap::<ResourceId, ComputedAggregate>::with_capacity(affected.len());
        for component_index in 0..components.len() {
            for id in &components[component_index] {
                let mut direct = None;
                let mut recursive =
                    cyclic[component_index].then_some(ResourceReadinessState::Failed);
                let mut fingerprint = DefaultHasher::new();
                if let Some(cycle_fingerprint) = cycle_fingerprints[component_index] {
                    cycle_fingerprint.hash(&mut fingerprint);
                }
                if let Some(source) = self.sources.get(id) {
                    for dependency in &source.record.dependency_ids {
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
                            let dependency_component = component_by_id.get(dependency).copied();
                            if dependency_component != Some(component_index) {
                                let nested = if dependency_component.is_some() {
                                    computed.get(dependency).copied()
                                } else {
                                    self.generation.row(*dependency).map(|previous| {
                                        ComputedAggregate {
                                            direct: previous.direct_dependency_state,
                                            recursive: previous.recursive_dependency_state,
                                            fingerprint: previous.dependency_fingerprint,
                                        }
                                    })
                                };
                                if let Some(nested) = nested {
                                    nested.fingerprint.hash(&mut fingerprint);
                                    recursive = combine_state(recursive, nested.recursive);
                                }
                            }
                        }
                    }
                }
                computed.insert(
                    *id,
                    ComputedAggregate {
                        direct: direct.unwrap_or(ResourceReadinessState::Loaded),
                        recursive: recursive.unwrap_or(ResourceReadinessState::Loaded),
                        fingerprint: fingerprint.finish(),
                    },
                );
            }
        }
        for id in ordered_affected {
            computed.entry(*id).or_insert(ComputedAggregate {
                direct: ResourceReadinessState::Loaded,
                recursive: ResourceReadinessState::Loaded,
                fingerprint: DefaultHasher::new().finish(),
            });
        }
        (computed, edge_visit_count)
    }
}

fn source_matches_update(
    current: Option<&ResourceReadinessSource>,
    update: &ResourceReadinessSourceUpdate,
) -> bool {
    match (current, update.record.as_ref()) {
        (Some(current), Some(record)) => {
            if current.runtime_state != update.runtime_state
                || current.payload_type_id != update.payload_type_id
            {
                return false;
            }
            if current.record.as_ref() == record {
                return true;
            }
            let mut canonical = record.clone();
            canonical.dependency_ids.sort_unstable();
            canonical.dependency_ids.dedup();
            current.record.as_ref() == &canonical
        }
        (None, None) => true,
        _ => false,
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

fn hash_cycle_component(component: &[ResourceId], hasher: &mut impl Hasher) {
    "resource-readiness-cycle".hash(hasher);
    component.hash(hasher);
}

#[cfg(test)]
mod tests;
