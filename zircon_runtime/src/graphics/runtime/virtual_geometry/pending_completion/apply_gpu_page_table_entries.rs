use std::collections::BTreeSet;

use super::super::{normalized_page_table_entries, VirtualGeometryRuntimeState};

impl VirtualGeometryRuntimeState {
    pub(crate) fn apply_gpu_page_table_entries(&mut self, page_table_entries: &[(u32, u32)]) {
        let unique_page_table_entries = normalized_page_table_entries(page_table_entries);

        let previous_resident_pages = self.resident_slots.keys().copied().collect::<BTreeSet<_>>();
        let previous_slot_owners = self
            .resident_slots
            .iter()
            .map(|(&page_id, &slot)| (slot, page_id))
            .collect::<Vec<_>>();
        let previous_hot_resident_pages = self.frontier_hot_resident_pages();
        let resident_page_ids = self.resident_slots.keys().copied().collect::<Vec<_>>();
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

        self.evictable_pages
            .retain(|page_id| self.resident_slots.contains_key(page_id));
        self.current_hot_resident_pages
            .retain(|page_id| self.resident_slots.contains_key(page_id));
        self.recent_hot_resident_pages
            .retain(|page_id, _| self.resident_slots.contains_key(page_id));
        self.current_hot_resident_pages.extend(
            inherited_hot_completed_pages(
                &unique_page_table_entries,
                &previous_resident_pages,
                &previous_slot_owners,
                &previous_hot_resident_pages,
                &surviving_previous_hot_resident_pages,
                &self.page_parent_pages,
            )
            .into_iter()
            .filter(|page_id| self.resident_slots.contains_key(page_id)),
        );
    }
}

fn inherited_hot_completed_pages(
    page_table_entries: &[(u32, u32)],
    previous_resident_pages: &BTreeSet<u32>,
    previous_slot_owners: &[(u32, u32)],
    previous_hot_resident_pages: &BTreeSet<u32>,
    surviving_previous_hot_resident_pages: &BTreeSet<u32>,
    page_parent_pages: &std::collections::BTreeMap<u32, u32>,
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
            if replaced_hot_page.is_some()
                || inherits_hot_ancestor(
                    *page_id,
                    surviving_previous_hot_resident_pages,
                    page_parent_pages,
                )
                || inherits_hot_descendant(
                    *page_id,
                    surviving_previous_hot_resident_pages,
                    page_parent_pages,
                )
            {
                return Some(*page_id);
            }

            None
        })
        .collect()
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

fn inherits_hot_descendant(
    page_id: u32,
    previous_hot_resident_pages: &BTreeSet<u32>,
    page_parent_pages: &std::collections::BTreeMap<u32, u32>,
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
