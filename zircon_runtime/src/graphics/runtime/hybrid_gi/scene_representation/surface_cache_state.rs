use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HybridGiSurfaceCachePageTableEntry {
    page_id: u32,
    atlas_slot_id: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HybridGiCardCaptureAtlasEntry {
    page_id: u32,
    capture_slot_id: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiSurfaceCacheState {
    resident_page_ids: Vec<u32>,
    dirty_page_ids: Vec<u32>,
    feedback_card_ids: Vec<u32>,
    invalidated_page_ids: Vec<u32>,
    page_table_entries: Vec<HybridGiSurfaceCachePageTableEntry>,
    capture_slot_reservations: Vec<HybridGiCardCaptureAtlasEntry>,
    capture_atlas_entries: Vec<HybridGiCardCaptureAtlasEntry>,
}

impl HybridGiSurfaceCacheState {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn synchronize(&mut self, active_card_ids: &[u32], page_budget: usize) {
        let previous_page_table_entries = self
            .page_table_entries
            .iter()
            .map(|entry| (entry.page_id, entry.atlas_slot_id))
            .collect::<std::collections::BTreeMap<_, _>>();
        let previous_capture_reservations = self
            .capture_slot_reservations
            .iter()
            .map(|entry| (entry.page_id, entry.capture_slot_id))
            .collect::<std::collections::BTreeMap<_, _>>();
        let previous_resident_ids = self.resident_page_ids.clone();
        let previous_resident_set = previous_resident_ids
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let mut next_resident_ids = active_card_ids
            .iter()
            .copied()
            .filter(|card_id| previous_resident_set.contains(card_id))
            .take(page_budget)
            .collect::<Vec<_>>();
        let next_resident_set = next_resident_ids.iter().copied().collect::<BTreeSet<_>>();

        self.invalidated_page_ids = previous_resident_ids
            .into_iter()
            .filter(|card_id| !next_resident_set.contains(card_id))
            .collect();
        self.dirty_page_ids.clear();
        self.feedback_card_ids.clear();

        for card_id in active_card_ids {
            if next_resident_set.contains(card_id) {
                continue;
            }

            if next_resident_ids.len() < page_budget {
                next_resident_ids.push(*card_id);
                self.dirty_page_ids.push(*card_id);
            } else {
                self.feedback_card_ids.push(*card_id);
            }
        }

        self.resident_page_ids = next_resident_ids;
        self.page_table_entries = assign_stable_slots(
            &self.resident_page_ids,
            &previous_page_table_entries,
            page_budget,
            |page_id, atlas_slot_id| HybridGiSurfaceCachePageTableEntry {
                page_id,
                atlas_slot_id,
            },
        );
        self.capture_slot_reservations = assign_stable_slots(
            &self.resident_page_ids,
            &previous_capture_reservations,
            self.resident_page_ids.len(),
            |page_id, capture_slot_id| HybridGiCardCaptureAtlasEntry {
                page_id,
                capture_slot_id,
            },
        );
        self.refresh_capture_atlas_entries();
    }

    pub(crate) fn mark_dirty_pages(&mut self, page_ids: impl IntoIterator<Item = u32>) {
        let resident_page_ids = self
            .resident_page_ids
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let mut dirty_page_ids = self.dirty_page_ids.iter().copied().collect::<BTreeSet<_>>();

        for page_id in page_ids {
            if resident_page_ids.contains(&page_id) {
                dirty_page_ids.insert(page_id);
            }
        }

        self.dirty_page_ids = dirty_page_ids.into_iter().collect();
        self.refresh_capture_atlas_entries();
    }

    pub(crate) fn mark_all_resident_pages_dirty(&mut self) {
        self.mark_dirty_pages(self.resident_page_ids.clone());
    }

    fn refresh_capture_atlas_entries(&mut self) {
        let dirty_page_ids = self.dirty_page_ids.iter().copied().collect::<BTreeSet<_>>();
        self.capture_atlas_entries = self
            .capture_slot_reservations
            .iter()
            .copied()
            .filter(|entry| dirty_page_ids.contains(&entry.page_id))
            .collect();
    }

    pub(crate) fn resident_page_count(&self) -> usize {
        self.resident_page_ids.len()
    }

    #[cfg(test)]
    pub(crate) fn resident_page_ids(&self) -> Vec<u32> {
        self.resident_page_ids.clone()
    }

    pub(crate) fn dirty_page_count(&self) -> usize {
        self.dirty_page_ids.len()
    }

    #[cfg(test)]
    pub(crate) fn dirty_page_ids(&self) -> Vec<u32> {
        self.dirty_page_ids.clone()
    }

    pub(crate) fn feedback_card_count(&self) -> usize {
        self.feedback_card_ids.len()
    }

    #[cfg(test)]
    pub(crate) fn feedback_card_ids(&self) -> Vec<u32> {
        self.feedback_card_ids.clone()
    }

    pub(crate) fn invalidated_page_count(&self) -> usize {
        self.invalidated_page_ids.len()
    }

    #[cfg(test)]
    pub(crate) fn invalidated_page_ids(&self) -> Vec<u32> {
        self.invalidated_page_ids.clone()
    }

    #[cfg(test)]
    pub(crate) fn page_table_entries(&self) -> Vec<(u32, u32)> {
        self.page_table_entries_snapshot()
    }

    pub(crate) fn page_table_entries_snapshot(&self) -> Vec<(u32, u32)> {
        self.page_table_entries
            .iter()
            .map(|entry| (entry.page_id, entry.atlas_slot_id))
            .collect()
    }

    pub(crate) fn capture_atlas_entries_snapshot(&self) -> Vec<(u32, u32)> {
        self.capture_atlas_entries
            .iter()
            .map(|entry| (entry.page_id, entry.capture_slot_id))
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn capture_slot_entries(&self) -> Vec<(u32, u32)> {
        self.capture_atlas_entries_snapshot()
    }
}

fn assign_stable_slots<Entry, F>(
    item_ids: &[u32],
    previous_slots: &std::collections::BTreeMap<u32, u32>,
    slot_capacity: usize,
    mut build_entry: F,
) -> Vec<Entry>
where
    F: FnMut(u32, u32) -> Entry,
{
    let mut assigned_slots = Vec::with_capacity(item_ids.len());
    let mut used_slots = BTreeSet::new();

    for &item_id in item_ids {
        let Some(previous_slot_id) = previous_slots.get(&item_id).copied() else {
            continue;
        };
        if previous_slot_id as usize >= slot_capacity || !used_slots.insert(previous_slot_id) {
            continue;
        }
        assigned_slots.push((item_id, previous_slot_id));
    }

    let mut free_slots = (0..slot_capacity as u32)
        .filter(|slot_id| !used_slots.contains(slot_id))
        .collect::<Vec<_>>()
        .into_iter();

    for &item_id in item_ids {
        if assigned_slots
            .iter()
            .any(|(assigned_item_id, _)| *assigned_item_id == item_id)
        {
            continue;
        }
        let Some(slot_id) = free_slots.next() else {
            break;
        };
        assigned_slots.push((item_id, slot_id));
    }

    assigned_slots
        .into_iter()
        .map(|(item_id, slot_id)| build_entry(item_id, slot_id))
        .collect()
}
