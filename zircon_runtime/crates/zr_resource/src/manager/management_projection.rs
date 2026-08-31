use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::sync::Arc;

use crate::{
    ResourceId, ResourceManagementGeneration, ResourceManagementGenerationDiagnostics,
    ResourceManagementIdShard, ResourceManagementLocatorShard, ResourceManagementRow,
    ResourceManagementSummary, ResourceRecord, resource_management_id_maps_from_ordered_pages,
    resource_management_pages_from_sorted_rows, resource_management_row_order,
};

#[derive(Debug)]
struct ProjectedResourceChange {
    previous: Option<Arc<ResourceManagementRow>>,
    next: Option<Arc<ResourceManagementRow>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResourceManagementOrderedStrategy {
    ReplacePages,
    RebalanceRanges,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResourceManagementIndexStrategy {
    Reuse,
    Sparse,
    Rebuild,
}

#[derive(Debug)]
struct ResourceManagementProjectionPlan {
    ordered_page_ranges: Vec<Range<usize>>,
    id_shard_indices: Vec<usize>,
    locator_shard_indices: Vec<usize>,
    ordered_strategy: ResourceManagementOrderedStrategy,
    id_index_strategy: ResourceManagementIndexStrategy,
    locator_index_strategy: ResourceManagementIndexStrategy,
}

#[derive(Debug)]
struct SparseResourceManagementPlan {
    ordered_page_ranges: Vec<Range<usize>>,
    id_shard_indices: Vec<usize>,
    locator_shard_indices: Vec<usize>,
    preserves_order: bool,
}

#[derive(Debug, Default)]
pub(super) struct ResourceManagementProjection {
    generation: Arc<ResourceManagementGeneration>,
}

impl ResourceManagementProjection {
    pub(super) fn generation(&self) -> Arc<ResourceManagementGeneration> {
        self.generation.clone()
    }

    pub(super) fn apply_delta<'a>(
        &mut self,
        removed_ids: impl IntoIterator<Item = ResourceId>,
        records: impl IntoIterator<Item = &'a ResourceRecord>,
    ) {
        let mut changes = HashMap::<ResourceId, ProjectedResourceChange>::new();
        for id in removed_ids {
            if let Some(previous) = self.generation.row_by_id(id) {
                changes.insert(
                    id,
                    ProjectedResourceChange {
                        previous: Some(previous),
                        next: None,
                    },
                );
            }
        }
        for record in records {
            let previous = self.generation.row_by_id(record.id);
            if previous
                .as_ref()
                .is_some_and(|row| resource_management_row_matches_record(row, record))
            {
                if !changes.is_empty() {
                    changes.remove(&record.id);
                }
            } else {
                changes.insert(
                    record.id,
                    ProjectedResourceChange {
                        next: Some(Arc::new(
                            ResourceManagementRow::from_record_reusing_identity(
                                record,
                                previous.as_deref(),
                            ),
                        )),
                        previous,
                    },
                );
            }
        }
        if changes.is_empty() {
            return;
        }
        self.generation = if self.generation.ordered_pages().is_empty() {
            rebuild_generation(&self.generation, &changes)
        } else {
            let plan = projection_plan(&self.generation, &changes);
            apply_projection_plan(&self.generation, &changes, plan)
        };
    }
}

fn projection_plan(
    generation: &ResourceManagementGeneration,
    changes: &HashMap<ResourceId, ProjectedResourceChange>,
) -> ResourceManagementProjectionPlan {
    let mut added_row_count = 0usize;
    let mut removed_row_count = 0usize;
    let mut locator_change_count = 0usize;
    for change in changes.values() {
        added_row_count = added_row_count.saturating_add(usize::from(
            change.previous.is_none() && change.next.is_some(),
        ));
        removed_row_count = removed_row_count.saturating_add(usize::from(
            change.previous.is_some() && change.next.is_none(),
        ));
        locator_change_count =
            locator_change_count.saturating_add(usize::from(locator_mapping_changes(change)));
    }
    let final_row_count = generation
        .summary()
        .total_count()
        .saturating_add(added_row_count)
        .saturating_sub(removed_row_count);
    let id_index_strategy = if dense_index_rebuild_is_cheaper(final_row_count, changes.len()) {
        ResourceManagementIndexStrategy::Rebuild
    } else {
        ResourceManagementIndexStrategy::Sparse
    };
    let locator_index_strategy = if locator_change_count == 0 {
        ResourceManagementIndexStrategy::Reuse
    } else if dense_index_rebuild_is_cheaper(final_row_count, locator_change_count) {
        ResourceManagementIndexStrategy::Rebuild
    } else {
        ResourceManagementIndexStrategy::Sparse
    };
    let ordered_plan = sparse_projection_plan(
        generation,
        changes,
        id_index_strategy,
        locator_index_strategy,
    );
    ResourceManagementProjectionPlan {
        ordered_page_ranges: ordered_plan.ordered_page_ranges,
        id_shard_indices: ordered_plan.id_shard_indices,
        locator_shard_indices: ordered_plan.locator_shard_indices,
        ordered_strategy: if ordered_plan.preserves_order {
            ResourceManagementOrderedStrategy::ReplacePages
        } else {
            ResourceManagementOrderedStrategy::RebalanceRanges
        },
        id_index_strategy,
        locator_index_strategy,
    }
}

fn dense_index_rebuild_is_cheaper(final_row_count: usize, changed_entry_count: usize) -> bool {
    changed_entry_count.saturating_mul(2) > final_row_count
}

fn sparse_projection_plan(
    generation: &ResourceManagementGeneration,
    changes: &HashMap<ResourceId, ProjectedResourceChange>,
    id_index_strategy: ResourceManagementIndexStrategy,
    locator_index_strategy: ResourceManagementIndexStrategy,
) -> SparseResourceManagementPlan {
    let mut page_indices = HashSet::new();
    let mut id_shard_indices = HashSet::new();
    let mut locator_shard_indices = HashSet::new();
    let preserves_order = changes.values().all(order_key_is_unchanged);
    for (id, change) in changes {
        if id_index_strategy == ResourceManagementIndexStrategy::Sparse {
            id_shard_indices.insert(generation.id_shard_index(*id));
        }
        if !preserves_order {
            for row in [change.previous.as_ref(), change.next.as_ref()]
                .into_iter()
                .flatten()
            {
                page_indices.insert(ordered_page_index(generation, row));
            }
        }
        if locator_index_strategy == ResourceManagementIndexStrategy::Sparse
            && locator_mapping_changes(change)
        {
            for row in [change.previous.as_ref(), change.next.as_ref()]
                .into_iter()
                .flatten()
            {
                locator_shard_indices.insert(generation.locator_shard_index(&row.primary_locator));
            }
        }
    }
    if preserves_order {
        page_indices = order_preserving_page_indices(generation, changes);
    }
    SparseResourceManagementPlan {
        ordered_page_ranges: if preserves_order {
            single_page_ranges(page_indices)
        } else {
            expanded_page_ranges(generation.ordered_pages().len(), page_indices)
        },
        id_shard_indices: sorted_indices(id_shard_indices),
        locator_shard_indices: sorted_indices(locator_shard_indices),
        preserves_order,
    }
}

fn apply_projection_plan(
    generation: &ResourceManagementGeneration,
    changes: &HashMap<ResourceId, ProjectedResourceChange>,
    plan: ResourceManagementProjectionPlan,
) -> Arc<ResourceManagementGeneration> {
    let mut summary = generation.summary().clone();
    apply_summary_delta(&mut summary, changes);
    let ordered_pages = match plan.ordered_strategy {
        ResourceManagementOrderedStrategy::ReplacePages => {
            replace_ordered_page_rows(generation, changes, &plan.ordered_page_ranges)
        }
        ResourceManagementOrderedStrategy::RebalanceRanges => {
            rebalance_ordered_page_ranges(generation, changes, &plan.ordered_page_ranges)
        }
    };
    let id_shards = match plan.id_index_strategy {
        ResourceManagementIndexStrategy::Reuse => generation.id_shards_arc(),
        ResourceManagementIndexStrategy::Sparse => {
            sparse_id_shards(generation, changes, &plan.id_shard_indices)
        }
        ResourceManagementIndexStrategy::Rebuild => rebuild_id_shards(generation, &ordered_pages),
    };
    let locator_shards = match plan.locator_index_strategy {
        ResourceManagementIndexStrategy::Reuse => generation.locator_shards_arc(),
        ResourceManagementIndexStrategy::Sparse => {
            sparse_locator_shards(generation, changes, &plan.locator_shard_indices)
        }
        ResourceManagementIndexStrategy::Rebuild => {
            rebuild_locator_shards(generation, &ordered_pages)
        }
    };
    Arc::new(ResourceManagementGeneration::from_parts(
        ResourceManagementGenerationDiagnostics {
            publication_count: generation.diagnostics().publication_count.saturating_add(1),
        },
        summary,
        generation.hash_authority_arc(),
        ordered_pages,
        id_shards,
        locator_shards,
    ))
}

fn replace_ordered_page_rows(
    generation: &ResourceManagementGeneration,
    changes: &HashMap<ResourceId, ProjectedResourceChange>,
    ranges: &[Range<usize>],
) -> Arc<[Arc<[Arc<ResourceManagementRow>]>]> {
    let copied_row_count = ranges
        .iter()
        .flat_map(|range| range.clone())
        .map(|index| generation.ordered_pages()[index].len())
        .sum::<usize>();
    let page_search_steps = generation.ordered_pages().len().next_power_of_two().ilog2() as usize;
    let row_search_steps = generation
        .ordered_pages()
        .iter()
        .map(|page| page.len())
        .max()
        .unwrap_or(1)
        .next_power_of_two()
        .ilog2() as usize;
    if changes
        .len()
        .saturating_mul(page_search_steps.saturating_add(row_search_steps))
        >= copied_row_count
    {
        let mut output = generation.ordered_pages().to_vec();
        for page_index in ranges.iter().map(|range| range.start) {
            let page = generation.ordered_pages()[page_index]
                .iter()
                .map(|row| {
                    changes
                        .get(&row.id)
                        .and_then(|change| change.next.as_ref())
                        .map_or_else(|| Arc::clone(row), Arc::clone)
                })
                .collect::<Vec<_>>();
            output[page_index] = page.into();
        }
        return output.into();
    }
    let mut changes_by_page = HashMap::<usize, Vec<&ProjectedResourceChange>>::new();
    for change in changes.values() {
        let previous = change
            .previous
            .as_ref()
            .expect("same-key replacement has a published row");
        changes_by_page
            .entry(ordered_page_index(generation, previous))
            .or_default()
            .push(change);
    }
    let mut output = generation.ordered_pages().to_vec();
    for page_index in ranges.iter().map(|range| range.start) {
        let mut page = generation.ordered_pages()[page_index].to_vec();
        for change in changes_by_page.remove(&page_index).unwrap_or_default() {
            let previous = change.previous.as_ref().unwrap();
            let row_index = page
                .binary_search_by(|row| resource_management_row_order(row, previous))
                .expect("same-key replacement remains in its published page");
            page[row_index] = Arc::clone(change.next.as_ref().unwrap());
        }
        output[page_index] = page.into();
    }
    output.into()
}

fn rebalance_ordered_page_ranges(
    generation: &ResourceManagementGeneration,
    changes: &HashMap<ResourceId, ProjectedResourceChange>,
    ranges: &[Range<usize>],
) -> Arc<[Arc<[Arc<ResourceManagementRow>]>]> {
    let mut structural_insertions_by_range = vec![Vec::new(); ranges.len()];
    for row in changes
        .values()
        .filter(|change| !order_key_is_unchanged(change))
        .filter_map(|change| change.next.as_ref())
    {
        let ordered_page_index = ordered_page_index(generation, row);
        let range_index = ranges.partition_point(|range| range.end <= ordered_page_index);
        let range = ranges
            .get(range_index)
            .expect("the sparse plan includes every structural insertion page");
        assert!(
            range.contains(&ordered_page_index),
            "the structural insertion page belongs to its planned range"
        );
        structural_insertions_by_range[range_index].push(Arc::clone(row));
    }
    let mut output = Vec::with_capacity(generation.ordered_pages().len());
    let mut page_index = 0usize;
    for (range, mut insertions) in ranges.iter().zip(structural_insertions_by_range) {
        output.extend(
            generation.ordered_pages()[page_index..range.start]
                .iter()
                .cloned(),
        );
        let mut rows = generation.ordered_pages()[range.clone()]
            .iter()
            .flat_map(|page| page.iter())
            .filter_map(|row| match changes.get(&row.id) {
                None => Some(Arc::clone(row)),
                Some(change) if order_key_is_unchanged(change) => {
                    change.next.as_ref().map(Arc::clone)
                }
                Some(_) => None,
            })
            .collect::<Vec<_>>();
        insertions.sort_by(|left, right| resource_management_row_order(left, right));
        rows = merge_ordered_rows(rows, insertions);
        output.extend(
            resource_management_pages_from_sorted_rows(rows)
                .iter()
                .cloned(),
        );
        page_index = range.end;
    }
    output.extend(generation.ordered_pages()[page_index..].iter().cloned());
    output.into()
}

fn sparse_id_shards(
    generation: &ResourceManagementGeneration,
    changes: &HashMap<ResourceId, ProjectedResourceChange>,
    changed_indices: &[usize],
) -> Arc<[Arc<ResourceManagementIdShard>]> {
    let mut changes_by_shard = (0..generation.id_shards().len())
        .map(|_| Vec::new())
        .collect::<Vec<_>>();
    for (id, change) in changes {
        changes_by_shard[generation.id_shard_index(*id)].push((*id, change));
    }
    let mut shards = generation.id_shards().to_vec();
    for index in changed_indices.iter().copied() {
        let mut entries = generation.id_shards()[index].entries().clone();
        for (id, change) in changes_by_shard[index].drain(..) {
            if let Some(next) = &change.next {
                entries.insert(id, Arc::clone(next));
            } else {
                entries.remove(&id);
            }
        }
        shards[index] = Arc::new(ResourceManagementIdShard::from_entries(entries));
    }
    shards.into()
}

fn rebuild_id_shards(
    generation: &ResourceManagementGeneration,
    ordered_pages: &[Arc<[Arc<ResourceManagementRow>]>],
) -> Arc<[Arc<ResourceManagementIdShard>]> {
    resource_management_id_maps_from_ordered_pages(
        ordered_pages,
        generation.hash_authority_arc().as_ref(),
    )
    .into_iter()
    .map(ResourceManagementIdShard::from_entries)
    .map(Arc::new)
    .collect::<Vec<_>>()
    .into()
}

fn sparse_locator_shards(
    generation: &ResourceManagementGeneration,
    changes: &HashMap<ResourceId, ProjectedResourceChange>,
    changed_indices: &[usize],
) -> Arc<[Arc<ResourceManagementLocatorShard>]> {
    let mut removals = changed_indices
        .iter()
        .copied()
        .map(|index| (index, Vec::new()))
        .collect::<HashMap<_, _>>();
    let mut insertions = changed_indices
        .iter()
        .copied()
        .map(|index| (index, Vec::new()))
        .collect::<HashMap<_, _>>();
    for change in changes
        .values()
        .filter(|change| locator_mapping_changes(change))
    {
        if let Some(previous) = &change.previous {
            removals
                .get_mut(&generation.locator_shard_index(&previous.primary_locator))
                .expect("the sparse plan includes every removed locator shard")
                .push(Arc::clone(&previous.primary_locator));
        }
        if let Some(next) = &change.next {
            insertions
                .get_mut(&generation.locator_shard_index(&next.primary_locator))
                .expect("the sparse plan includes every inserted locator shard")
                .push((Arc::clone(&next.primary_locator), next.id));
        }
    }
    let mut shards = generation.locator_shards().to_vec();
    for index in changed_indices.iter().copied() {
        let mut entries = generation.locator_shards()[index].entries().clone();
        for locator in removals.remove(&index).unwrap_or_default() {
            entries.remove(locator.as_ref());
        }
        for (locator, id) in insertions.remove(&index).unwrap_or_default() {
            entries.insert(locator, id);
        }
        shards[index] = Arc::new(ResourceManagementLocatorShard::from_entries(entries));
    }
    shards.into()
}

fn rebuild_locator_shards(
    generation: &ResourceManagementGeneration,
    ordered_pages: &[Arc<[Arc<ResourceManagementRow>]>],
) -> Arc<[Arc<ResourceManagementLocatorShard>]> {
    let mut entries = (0..generation.locator_shards().len())
        .map(|_| HashMap::new())
        .collect::<Vec<_>>();
    for row in ordered_pages.iter().flat_map(|page| page.iter()) {
        entries[generation.locator_shard_index(&row.primary_locator)]
            .insert(Arc::clone(&row.primary_locator), row.id);
    }
    entries
        .into_iter()
        .map(ResourceManagementLocatorShard::from_entries)
        .map(Arc::new)
        .collect::<Vec<_>>()
        .into()
}

fn rebuild_generation(
    generation: &ResourceManagementGeneration,
    changes: &HashMap<ResourceId, ProjectedResourceChange>,
) -> Arc<ResourceManagementGeneration> {
    let final_count = generation
        .summary()
        .total_count()
        .saturating_add(
            changes
                .values()
                .filter(|change| change.previous.is_none() && change.next.is_some())
                .count(),
        )
        .saturating_sub(
            changes
                .values()
                .filter(|change| change.previous.is_some() && change.next.is_none())
                .count(),
        );
    let mut ordered = Vec::with_capacity(final_count);
    let mut insertions = Vec::new();
    for row in generation.ordered_rows() {
        match changes.get(&row.id) {
            None => ordered.push(Arc::clone(row)),
            Some(change) => match &change.next {
                Some(next) if resource_management_row_order(row, next).is_eq() => {
                    ordered.push(Arc::clone(next));
                }
                Some(next) => insertions.push(Arc::clone(next)),
                None => {}
            },
        }
    }
    insertions.extend(changes.values().filter_map(|change| {
        (change.previous.is_none())
            .then(|| change.next.as_ref())
            .flatten()
            .cloned()
    }));
    insertions.sort_by(|left, right| resource_management_row_order(left, right));
    let ordered = merge_ordered_rows(ordered, insertions);
    debug_assert_eq!(ordered.len(), final_count);
    Arc::new(
        ResourceManagementGeneration::from_sorted_rows_with_hash_authority(
            ResourceManagementGenerationDiagnostics {
                publication_count: generation.diagnostics().publication_count.saturating_add(1),
            },
            ordered,
            generation.hash_authority_arc(),
        ),
    )
}

fn merge_ordered_rows(
    existing: Vec<Arc<ResourceManagementRow>>,
    insertions: Vec<Arc<ResourceManagementRow>>,
) -> Vec<Arc<ResourceManagementRow>> {
    let mut merged = Vec::with_capacity(existing.len().saturating_add(insertions.len()));
    let mut existing = existing.into_iter().peekable();
    let mut insertions = insertions.into_iter().peekable();
    while let (Some(left), Some(right)) = (existing.peek(), insertions.peek()) {
        if resource_management_row_order(left, right).is_le() {
            merged.push(existing.next().unwrap());
        } else {
            merged.push(insertions.next().unwrap());
        }
    }
    merged.extend(existing);
    merged.extend(insertions);
    merged
}

fn apply_summary_delta(
    summary: &mut ResourceManagementSummary,
    changes: &HashMap<ResourceId, ProjectedResourceChange>,
) {
    for change in changes.values() {
        if !summary_mapping_changes(change) {
            continue;
        }
        if let Some(previous) = &change.previous {
            summary.remove(previous);
        }
        if let Some(next) = &change.next {
            summary.add(next);
        }
    }
}

fn ordered_page_index(
    generation: &ResourceManagementGeneration,
    row: &ResourceManagementRow,
) -> usize {
    let pages = generation.ordered_pages();
    let index = pages
        .partition_point(|page| resource_management_row_order(page.last().unwrap(), row).is_lt());
    index.min(pages.len().saturating_sub(1))
}

fn expanded_page_ranges(page_count: usize, page_indices: HashSet<usize>) -> Vec<Range<usize>> {
    let mut ranges = page_indices
        .into_iter()
        .map(|index| index.saturating_sub(1)..(index + 2).min(page_count))
        .collect::<Vec<_>>();
    ranges.sort_by_key(|range| range.start);
    let mut merged = Vec::<Range<usize>>::new();
    for range in ranges {
        if let Some(previous) = merged.last_mut() {
            if range.start <= previous.end {
                previous.end = previous.end.max(range.end);
                continue;
            }
        }
        merged.push(range);
    }
    merged
}

fn order_preserving_page_indices(
    generation: &ResourceManagementGeneration,
    changes: &HashMap<ResourceId, ProjectedResourceChange>,
) -> HashSet<usize> {
    let page_search_steps = generation.ordered_pages().len().next_power_of_two().ilog2() as usize;
    if changes.len().saturating_mul(page_search_steps) < generation.summary().total_count() {
        return changes
            .values()
            .filter_map(|change| change.previous.as_ref())
            .map(|row| ordered_page_index(generation, row))
            .collect();
    }
    if changes.len().saturating_mul(2) > generation.summary().total_count() {
        return (0..generation.ordered_pages().len()).collect();
    }
    generation
        .ordered_pages()
        .iter()
        .enumerate()
        .filter_map(|(index, page)| {
            page.iter()
                .any(|row| changes.contains_key(&row.id))
                .then_some(index)
        })
        .collect()
}

fn single_page_ranges(page_indices: HashSet<usize>) -> Vec<Range<usize>> {
    let mut indices = page_indices.into_iter().collect::<Vec<_>>();
    indices.sort_unstable();
    indices
        .into_iter()
        .map(|index| index..index.saturating_add(1))
        .collect()
}

fn sorted_indices(indices: HashSet<usize>) -> Vec<usize> {
    let mut indices = indices.into_iter().collect::<Vec<_>>();
    indices.sort_unstable();
    indices
}

fn locator_mapping_changes(change: &ProjectedResourceChange) -> bool {
    match (&change.previous, &change.next) {
        (Some(previous), Some(next)) => previous.primary_locator != next.primary_locator,
        (None, None) => false,
        _ => true,
    }
}

fn order_key_is_unchanged(change: &ProjectedResourceChange) -> bool {
    matches!(
        (&change.previous, &change.next),
        (Some(previous), Some(next))
            if previous.primary_locator == next.primary_locator && previous.id == next.id
    )
}

fn summary_mapping_changes(change: &ProjectedResourceChange) -> bool {
    !matches!(
        (&change.previous, &change.next),
        (Some(previous), Some(next)) if previous.kind == next.kind && previous.state == next.state
    )
}

fn resource_management_row_matches_record(
    row: &ResourceManagementRow,
    record: &ResourceRecord,
) -> bool {
    row.id == record.id
        && row.kind == record.kind
        && record
            .primary_locator
            .matches_display(row.primary_locator.as_ref())
        && row.revision == record.revision
        && row.state == record.state
        && row.diagnostic_count == record.diagnostics.len()
}

#[cfg(test)]
mod tests;
