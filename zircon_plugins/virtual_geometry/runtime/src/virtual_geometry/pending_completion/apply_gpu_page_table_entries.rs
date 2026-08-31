use std::collections::{BTreeMap, BTreeSet};

use super::super::{normalized_page_table_entries, VirtualGeometryRuntimeState};

impl VirtualGeometryRuntimeState {
    pub(crate) fn apply_gpu_page_table_entries(&mut self, page_table_entries: &[(u32, u32)]) {
        let unique_page_table_entries = normalized_page_table_entries(page_table_entries);

        let previous_resident_pages = self.resident_page_ids().collect::<BTreeSet<_>>();
        let previous_page_by_slot = self.resident_page_slots().fold(
            BTreeMap::new(),
            |mut page_by_slot, (page_id, slot)| {
                page_by_slot.entry(slot).or_insert(page_id);
                page_by_slot
            },
        );
        let previous_hot_resident_pages = self.frontier_hot_resident_pages();
        let resident_page_ids = self.resident_page_ids().collect::<Vec<_>>();
        let gpu_resident_pages = unique_page_table_entries
            .iter()
            .map(|(page_id, _)| *page_id)
            .collect::<BTreeSet<_>>();
        let surviving_previous_hot_resident_pages = previous_hot_resident_pages
            .iter()
            .copied()
            .filter(|page_id| gpu_resident_pages.contains(page_id))
            .collect::<BTreeSet<_>>();

        for page_id in resident_page_ids {
            if !gpu_resident_pages.contains(&page_id) {
                self.evict_page(page_id);
            }
        }

        for (page_id, slot) in &unique_page_table_entries {
            self.promote_to_resident_in_slot(*page_id, *slot);
        }

        self.retain_resident_evictable_pages();
        let resident_page_ids = self.resident_page_ids().collect::<BTreeSet<_>>();
        self.retain_current_hot_resident_pages(|page_id| resident_page_ids.contains(page_id));
        self.retain_recent_hot_resident_pages(|page_id, _| resident_page_ids.contains(page_id));
        let inherited_hot_completed_pages = indexed_inherited_hot_completed_pages(
            &unique_page_table_entries,
            &previous_resident_pages,
            &previous_page_by_slot,
            &previous_hot_resident_pages,
            &surviving_previous_hot_resident_pages,
            self.page_parent_pages(),
        )
        .into_iter()
        .filter(|page_id| resident_page_ids.contains(page_id))
        .collect::<Vec<_>>();
        self.extend_current_hot_resident_pages(inherited_hot_completed_pages);
    }
}

fn indexed_inherited_hot_completed_pages(
    page_table_entries: &[(u32, u32)],
    previous_resident_pages: &BTreeSet<u32>,
    previous_page_by_slot: &BTreeMap<u32, u32>,
    previous_hot_resident_pages: &BTreeSet<u32>,
    surviving_previous_hot_resident_pages: &BTreeSet<u32>,
    page_parent_pages: &BTreeMap<u32, u32>,
) -> BTreeSet<u32> {
    let hot_ancestor_page_ids =
        hot_ancestor_page_ids(surviving_previous_hot_resident_pages, page_parent_pages);
    page_table_entries
        .iter()
        .filter_map(|(page_id, slot)| {
            if previous_resident_pages.contains(page_id) {
                return None;
            }

            let replaced_hot_page =
                previous_page_by_slot
                    .get(slot)
                    .copied()
                    .filter(|previous_page_id| {
                        *previous_page_id != *page_id
                            && previous_hot_resident_pages.contains(previous_page_id)
                    });
            if replaced_hot_page.is_some()
                || inherits_hot_ancestor(
                    *page_id,
                    surviving_previous_hot_resident_pages,
                    page_parent_pages,
                )
                || hot_ancestor_page_ids.contains(page_id)
            {
                return Some(*page_id);
            }

            None
        })
        .collect()
}

fn hot_ancestor_page_ids(
    hot_page_ids: &BTreeSet<u32>,
    page_parent_pages: &BTreeMap<u32, u32>,
) -> BTreeSet<u32> {
    let mut hot_ancestor_page_ids = BTreeSet::new();
    for &hot_page_id in hot_page_ids {
        let mut current_page_id = hot_page_id;
        while let Some(parent_page_id) = page_parent_pages.get(&current_page_id).copied() {
            if !hot_ancestor_page_ids.insert(parent_page_id) {
                break;
            }
            current_page_id = parent_page_id;
        }
    }
    hot_ancestor_page_ids
}

fn inherits_hot_ancestor(
    page_id: u32,
    previous_hot_resident_pages: &BTreeSet<u32>,
    page_parent_pages: &std::collections::BTreeMap<u32, u32>,
) -> bool {
    let mut current_page_id = page_id;
    while let Some(parent_page_id) = page_parent_pages.get(&current_page_id).copied() {
        if previous_hot_resident_pages.contains(&parent_page_id) {
            return true;
        }
        current_page_id = parent_page_id;
    }

    false
}

#[cfg(test)]
fn inherits_hot_descendant(
    page_id: u32,
    previous_hot_resident_pages: &BTreeSet<u32>,
    page_parent_pages: &BTreeMap<u32, u32>,
) -> bool {
    let mut stack = page_parent_pages
        .iter()
        .filter_map(|(&candidate_page_id, &parent_page_id)| {
            (parent_page_id == page_id).then_some(candidate_page_id)
        })
        .collect::<Vec<_>>();
    let mut visited_page_ids = BTreeSet::new();

    while let Some(candidate_page_id) = stack.pop() {
        if !visited_page_ids.insert(candidate_page_id) {
            continue;
        }
        if previous_hot_resident_pages.contains(&candidate_page_id) {
            return true;
        }
        stack.extend(page_parent_pages.iter().filter_map(
            |(&descendant_page_id, &parent_page_id)| {
                (parent_page_id == candidate_page_id).then_some(descendant_page_id)
            },
        ));
    }

    false
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const BENCH_CANDIDATE_PAGES: usize = 2_048;
    const BENCH_HIERARCHY_EDGES: usize = 4_096;
    const BENCH_HOT_PAGES: usize = 64;
    const BENCH_SAMPLE_PAIRS: usize = 21;

    #[test]
    fn indexed_inheritance_matches_replacement_ancestor_and_descendant_semantics() {
        let page_table_entries = vec![(2, 0), (3, 3), (1, 4), (100, 2), (200, 5)];
        let previous_resident_pages = BTreeSet::from([2, 20, 99]);
        let previous_page_by_slot = BTreeMap::from([(0, 2), (1, 20), (2, 99)]);
        let previous_slot_owners = previous_page_by_slot
            .iter()
            .map(|(&slot, &page_id)| (slot, page_id))
            .collect::<Vec<_>>();
        let previous_hot_resident_pages = BTreeSet::from([2, 20, 99]);
        let surviving_previous_hot_resident_pages = BTreeSet::from([2, 20]);
        let page_parent_pages = BTreeMap::from([(2, 1), (3, 2), (20, 10), (21, 20)]);

        let indexed = indexed_inherited_hot_completed_pages(
            &page_table_entries,
            &previous_resident_pages,
            &previous_page_by_slot,
            &previous_hot_resident_pages,
            &surviving_previous_hot_resident_pages,
            &page_parent_pages,
        );
        let legacy = legacy_inherited_hot_completed_pages(
            &page_table_entries,
            &previous_resident_pages,
            &previous_slot_owners,
            &previous_hot_resident_pages,
            &surviving_previous_hot_resident_pages,
            &page_parent_pages,
        );

        assert_eq!(indexed, BTreeSet::from([1, 3, 100]));
        assert_eq!(indexed, legacy);
    }

    #[test]
    #[ignore = "release performance gate"]
    fn indexed_hot_inheritance_release_benchmark() {
        let hierarchy_first_hot_page = BENCH_HIERARCHY_EDGES - BENCH_HOT_PAGES + 1;
        let page_parent_pages = (1..=BENCH_HIERARCHY_EDGES)
            .map(|page_id| (page_id as u32, page_id.saturating_sub(1) as u32))
            .collect::<BTreeMap<_, _>>();
        let hot_pages = (hierarchy_first_hot_page..=BENCH_HIERARCHY_EDGES)
            .map(|page_id| page_id as u32)
            .collect::<BTreeSet<_>>();
        let previous_resident_pages = hot_pages.clone();
        let resident_page_slots = hot_pages
            .iter()
            .copied()
            .enumerate()
            .map(|(slot, page_id)| (page_id, slot as u32))
            .collect::<Vec<_>>();
        let previous_page_by_slot = resident_page_slots
            .iter()
            .copied()
            .map(|(page_id, slot)| (slot, page_id))
            .collect::<BTreeMap<_, _>>();
        let previous_slot_owners = resident_page_slots
            .iter()
            .copied()
            .map(|(page_id, slot)| (slot, page_id))
            .collect::<Vec<_>>();
        let mut page_table_entries = resident_page_slots.clone();
        page_table_entries.extend((0..BENCH_CANDIDATE_PAGES).map(|index| {
            (
                1_000_000_u32 + index as u32,
                BENCH_HOT_PAGES as u32 + index as u32,
            )
        }));

        assert_eq!(
            legacy_inherited_hot_completed_pages(
                &page_table_entries,
                &previous_resident_pages,
                &previous_slot_owners,
                &hot_pages,
                &hot_pages,
                &page_parent_pages,
            ),
            indexed_inherited_hot_completed_pages(
                &page_table_entries,
                &previous_resident_pages,
                &previous_page_by_slot,
                &hot_pages,
                &hot_pages,
                &page_parent_pages,
            )
        );

        let mut legacy_samples = Vec::with_capacity(BENCH_SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(BENCH_SAMPLE_PAIRS);
        for pair_index in 0..BENCH_SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_legacy(
                    &page_table_entries,
                    &previous_resident_pages,
                    &resident_page_slots,
                    &hot_pages,
                    &page_parent_pages,
                ));
                optimized_samples.push(measure_indexed(
                    &page_table_entries,
                    &previous_resident_pages,
                    &resident_page_slots,
                    &hot_pages,
                    &page_parent_pages,
                ));
            } else {
                optimized_samples.push(measure_indexed(
                    &page_table_entries,
                    &previous_resident_pages,
                    &resident_page_slots,
                    &hot_pages,
                    &page_parent_pages,
                ));
                legacy_samples.push(measure_legacy(
                    &page_table_entries,
                    &previous_resident_pages,
                    &resident_page_slots,
                    &hot_pages,
                    &page_parent_pages,
                ));
            }
        }

        let legacy_p50 = nearest_rank(&legacy_samples, 50);
        let legacy_p95 = nearest_rank(&legacy_samples, 95);
        let optimized_p50 = nearest_rank(&optimized_samples, 50);
        let optimized_p95 = nearest_rank(&optimized_samples, 95);
        let legacy_descendant_parent_scans = BENCH_CANDIDATE_PAGES * BENCH_HIERARCHY_EDGES;
        let legacy_linear_slot_owner_scans = BENCH_CANDIDATE_PAGES * BENCH_HOT_PAGES;
        let optimized_hot_ancestor_entries = BENCH_HIERARCHY_EDGES;
        let optimized_hot_ancestor_parent_lookups =
            hierarchy_first_hot_page + 1 + (BENCH_HOT_PAGES - 1) * 2;
        println!(
            "VIRTUAL_GEOMETRY_HOT_INHERITANCE_BENCH_V1 sample_pairs={} sample_order=alternating percentile_method=nearest_rank page_table_entries={} candidate_pages={} hierarchy_edges={} hot_pages={} legacy_descendant_parent_scans={} optimized_descendant_parent_scans=0 legacy_linear_slot_owner_scans={} optimized_indexed_slot_owner_lookups={} optimized_hot_ancestor_entries={} optimized_hot_ancestor_parent_lookups={} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={} optimized_ns={}",
            BENCH_SAMPLE_PAIRS,
            page_table_entries.len(),
            BENCH_CANDIDATE_PAGES,
            BENCH_HIERARCHY_EDGES,
            BENCH_HOT_PAGES,
            legacy_descendant_parent_scans,
            legacy_linear_slot_owner_scans,
            BENCH_CANDIDATE_PAGES,
            optimized_hot_ancestor_entries,
            optimized_hot_ancestor_parent_lookups,
            legacy_p50,
            legacy_p95,
            optimized_p50,
            optimized_p95,
            join_samples(&legacy_samples),
            join_samples(&optimized_samples),
        );
        assert!(
            optimized_p95.saturating_mul(4) <= legacy_p95,
            "indexed hot inheritance P95 must be at most 25% of legacy: legacy={legacy_p95}ns optimized={optimized_p95}ns"
        );
    }

    fn measure_legacy(
        page_table_entries: &[(u32, u32)],
        previous_resident_pages: &BTreeSet<u32>,
        resident_page_slots: &[(u32, u32)],
        hot_pages: &BTreeSet<u32>,
        page_parent_pages: &BTreeMap<u32, u32>,
    ) -> u128 {
        let started = Instant::now();
        let previous_slot_owners = black_box(resident_page_slots)
            .iter()
            .copied()
            .map(|(page_id, slot)| (slot, page_id))
            .collect::<Vec<_>>();
        black_box(legacy_inherited_hot_completed_pages(
            black_box(page_table_entries),
            black_box(previous_resident_pages),
            black_box(&previous_slot_owners),
            black_box(hot_pages),
            black_box(hot_pages),
            black_box(page_parent_pages),
        ));
        started.elapsed().as_nanos()
    }

    fn measure_indexed(
        page_table_entries: &[(u32, u32)],
        previous_resident_pages: &BTreeSet<u32>,
        resident_page_slots: &[(u32, u32)],
        hot_pages: &BTreeSet<u32>,
        page_parent_pages: &BTreeMap<u32, u32>,
    ) -> u128 {
        let started = Instant::now();
        let previous_page_by_slot = black_box(resident_page_slots).iter().copied().fold(
            BTreeMap::new(),
            |mut page_by_slot, (page_id, slot)| {
                page_by_slot.entry(slot).or_insert(page_id);
                page_by_slot
            },
        );
        black_box(indexed_inherited_hot_completed_pages(
            black_box(page_table_entries),
            black_box(previous_resident_pages),
            black_box(&previous_page_by_slot),
            black_box(hot_pages),
            black_box(hot_pages),
            black_box(page_parent_pages),
        ));
        started.elapsed().as_nanos()
    }

    fn legacy_inherited_hot_completed_pages(
        page_table_entries: &[(u32, u32)],
        previous_resident_pages: &BTreeSet<u32>,
        previous_slot_owners: &[(u32, u32)],
        previous_hot_resident_pages: &BTreeSet<u32>,
        surviving_previous_hot_resident_pages: &BTreeSet<u32>,
        page_parent_pages: &BTreeMap<u32, u32>,
    ) -> BTreeSet<u32> {
        page_table_entries
            .iter()
            .filter_map(|(page_id, slot)| {
                if previous_resident_pages.contains(page_id) {
                    return None;
                }
                let replaced_hot_page =
                    previous_slot_owners
                        .iter()
                        .find_map(|(previous_slot, previous_page_id)| {
                            (*previous_slot == *slot
                                && *previous_page_id != *page_id
                                && previous_hot_resident_pages.contains(previous_page_id))
                            .then_some(*previous_page_id)
                        });
                (replaced_hot_page.is_some()
                    || inherits_hot_ancestor(
                        *page_id,
                        surviving_previous_hot_resident_pages,
                        page_parent_pages,
                    )
                    || inherits_hot_descendant(
                        *page_id,
                        surviving_previous_hot_resident_pages,
                        page_parent_pages,
                    ))
                .then_some(*page_id)
            })
            .collect()
    }

    fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
        let mut ordered = samples.to_vec();
        ordered.sort_unstable();
        let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
        ordered[rank.saturating_sub(1)]
    }

    fn join_samples(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
