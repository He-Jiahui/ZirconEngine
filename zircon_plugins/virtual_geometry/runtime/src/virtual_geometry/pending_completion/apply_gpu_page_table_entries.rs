use std::collections::{BTreeMap, BTreeSet};

use super::super::{normalized_page_table_entries, VirtualGeometryRuntimeState};

impl VirtualGeometryRuntimeState {
    /// 以规范化 GPU 页表作为最终驻留事实，移除旧页并继承仍有效的热点。
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
#[path = "tests/apply_gpu_page_table_entries.rs"]
mod tests;
