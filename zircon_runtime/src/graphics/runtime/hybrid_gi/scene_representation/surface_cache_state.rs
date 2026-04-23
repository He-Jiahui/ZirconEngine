use std::collections::{BTreeMap, BTreeSet};

use crate::graphics::scene::HybridGiScenePrepareResourcesSnapshot;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HybridGiResidentSurfaceCachePageEntry {
    page_id: u32,
    owner_card_id: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HybridGiSurfaceCachePageContentEntry {
    page_id: u32,
    owner_card_id: u32,
    atlas_slot_id: u32,
    capture_slot_id: u32,
    atlas_sample_rgba: [u8; 4],
    capture_sample_rgba: [u8; 4],
}

fn rgba_sample_is_present(rgba: [u8; 4]) -> bool {
    rgba[3] > 0
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiSurfaceCacheState {
    resident_pages: Vec<HybridGiResidentSurfaceCachePageEntry>,
    dirty_page_ids: Vec<u32>,
    feedback_card_ids: Vec<u32>,
    invalidated_page_ids: Vec<u32>,
    page_table_entries: Vec<HybridGiSurfaceCachePageTableEntry>,
    capture_slot_reservations: Vec<HybridGiCardCaptureAtlasEntry>,
    capture_atlas_entries: Vec<HybridGiCardCaptureAtlasEntry>,
    page_contents: Vec<HybridGiSurfaceCachePageContentEntry>,
    next_page_id: u32,
    scene_revision: u32,
}

impl HybridGiSurfaceCacheState {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn synchronize(&mut self, active_card_ids: &[u32], page_budget: usize) {
        let previous_resident_pages_snapshot = self.resident_pages.clone();
        let previous_dirty_page_ids = self.dirty_page_ids.clone();
        let previous_feedback_card_ids = self.feedback_card_ids.clone();
        let previous_invalidated_page_ids = self.invalidated_page_ids.clone();
        let previous_page_table_entries_snapshot = self.page_table_entries.clone();
        let previous_capture_slot_reservations = self.capture_slot_reservations.clone();
        let previous_capture_atlas_entries = self.capture_atlas_entries.clone();
        let previous_page_contents = self.page_contents.clone();
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
        let previous_resident_pages = self.resident_pages.clone();
        let previous_resident_pages_by_owner_card_id = previous_resident_pages
            .iter()
            .copied()
            .map(|entry| (entry.owner_card_id, entry))
            .collect::<BTreeMap<_, _>>();
        let mut next_resident_pages = active_card_ids
            .iter()
            .filter_map(|owner_card_id| {
                previous_resident_pages_by_owner_card_id
                    .get(owner_card_id)
                    .copied()
            })
            .take(page_budget)
            .collect::<Vec<_>>();
        let next_resident_page_ids = next_resident_pages
            .iter()
            .map(|entry| entry.page_id)
            .collect::<BTreeSet<_>>();

        self.invalidated_page_ids = previous_resident_pages
            .into_iter()
            .map(|entry| entry.page_id)
            .filter(|page_id| !next_resident_page_ids.contains(page_id))
            .collect();
        self.dirty_page_ids.clear();
        self.feedback_card_ids.clear();
        let mut reusable_page_ids = self
            .invalidated_page_ids
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let next_resident_owner_card_ids = next_resident_pages
            .iter()
            .map(|entry| entry.owner_card_id)
            .collect::<BTreeSet<_>>();

        for owner_card_id in active_card_ids {
            if next_resident_owner_card_ids.contains(owner_card_id) {
                continue;
            }

            if next_resident_pages.len() < page_budget {
                let page_id = allocate_page_id(&mut reusable_page_ids, &mut self.next_page_id);
                next_resident_pages.push(HybridGiResidentSurfaceCachePageEntry {
                    page_id,
                    owner_card_id: *owner_card_id,
                });
                self.dirty_page_ids.push(page_id);
            } else {
                self.feedback_card_ids.push(*owner_card_id);
            }
        }

        self.resident_pages = next_resident_pages;
        self.page_table_entries = assign_stable_slots(
            &self.resident_page_ids_snapshot(),
            &previous_page_table_entries,
            page_budget,
            |page_id, atlas_slot_id| HybridGiSurfaceCachePageTableEntry {
                page_id,
                atlas_slot_id,
            },
        );
        self.capture_slot_reservations = assign_stable_slots(
            &self.resident_page_ids_snapshot(),
            &previous_capture_reservations,
            self.resident_pages.len(),
            |page_id, capture_slot_id| HybridGiCardCaptureAtlasEntry {
                page_id,
                capture_slot_id,
            },
        );
        self.refresh_capture_atlas_entries();
        self.rebind_page_contents();
        self.bump_scene_revision_if(
            self.resident_pages != previous_resident_pages_snapshot
                || self.dirty_page_ids != previous_dirty_page_ids
                || self.feedback_card_ids != previous_feedback_card_ids
                || self.invalidated_page_ids != previous_invalidated_page_ids
                || self.page_table_entries != previous_page_table_entries_snapshot
                || self.capture_slot_reservations != previous_capture_slot_reservations
                || self.capture_atlas_entries != previous_capture_atlas_entries
                || self.page_contents != previous_page_contents,
        );
    }

    pub(crate) fn mark_dirty_pages(&mut self, page_ids: impl IntoIterator<Item = u32>) {
        let previous_dirty_page_ids = self.dirty_page_ids.clone();
        let previous_capture_atlas_entries = self.capture_atlas_entries.clone();
        let resident_page_ids = self
            .resident_page_ids_snapshot()
            .into_iter()
            .collect::<BTreeSet<_>>();
        let mut dirty_page_ids = self.dirty_page_ids.iter().copied().collect::<BTreeSet<_>>();

        for page_id in page_ids {
            if resident_page_ids.contains(&page_id) {
                dirty_page_ids.insert(page_id);
            }
        }

        self.dirty_page_ids = dirty_page_ids.into_iter().collect();
        self.refresh_capture_atlas_entries();
        self.bump_scene_revision_if(
            self.dirty_page_ids != previous_dirty_page_ids
                || self.capture_atlas_entries != previous_capture_atlas_entries,
        );
    }

    pub(crate) fn mark_dirty_owner_cards(&mut self, owner_card_ids: impl IntoIterator<Item = u32>) {
        let page_ids_by_owner_card_id = self
            .resident_pages
            .iter()
            .map(|entry| (entry.owner_card_id, entry.page_id))
            .collect::<BTreeMap<_, _>>();
        self.mark_dirty_pages(
            owner_card_ids
                .into_iter()
                .filter_map(|owner_card_id| page_ids_by_owner_card_id.get(&owner_card_id).copied()),
        );
    }

    pub(crate) fn mark_all_resident_pages_dirty(&mut self) {
        self.mark_dirty_pages(self.resident_page_ids_snapshot());
    }

    pub(crate) fn apply_scene_prepare_resources(
        &mut self,
        snapshot: &HybridGiScenePrepareResourcesSnapshot,
    ) {
        let previous_page_contents = self.page_contents.clone();
        let atlas_samples_by_slot = snapshot
            .atlas_slot_rgba_samples
            .iter()
            .copied()
            .collect::<BTreeMap<_, _>>();
        let capture_samples_by_slot = snapshot
            .capture_slot_rgba_samples
            .iter()
            .copied()
            .collect::<BTreeMap<_, _>>();
        let existing_entries = self
            .page_contents
            .iter()
            .copied()
            .map(|entry| (entry.page_id, entry))
            .collect::<BTreeMap<_, _>>();
        let atlas_slots_by_page_id = self
            .page_table_entries
            .iter()
            .map(|entry| (entry.page_id, entry.atlas_slot_id))
            .collect::<BTreeMap<_, _>>();
        let capture_slots_by_page_id = self
            .capture_slot_reservations
            .iter()
            .map(|entry| (entry.page_id, entry.capture_slot_id))
            .collect::<BTreeMap<_, _>>();
        let owner_card_ids_by_page_id = self
            .resident_pages
            .iter()
            .map(|entry| (entry.page_id, entry.owner_card_id))
            .collect::<BTreeMap<_, _>>();

        self.page_contents = self
            .resident_pages
            .iter()
            .filter_map(|page| {
                let atlas_slot_id = atlas_slots_by_page_id.get(&page.page_id).copied()?;
                let capture_slot_id = capture_slots_by_page_id.get(&page.page_id).copied()?;
                let existing = existing_entries
                    .get(&page.page_id)
                    .copied()
                    .filter(|entry| entry.owner_card_id == page.owner_card_id);
                let atlas_sample_rgba = atlas_samples_by_slot
                    .get(&atlas_slot_id)
                    .copied()
                    .or_else(|| existing.map(|entry| entry.atlas_sample_rgba))
                    .unwrap_or([0, 0, 0, 0]);
                let capture_sample_rgba = capture_samples_by_slot
                    .get(&capture_slot_id)
                    .copied()
                    .or_else(|| existing.map(|entry| entry.capture_sample_rgba))
                    .unwrap_or([0, 0, 0, 0]);

                if !rgba_sample_is_present(atlas_sample_rgba)
                    && !rgba_sample_is_present(capture_sample_rgba)
                {
                    return None;
                }

                Some(HybridGiSurfaceCachePageContentEntry {
                    page_id: page.page_id,
                    owner_card_id: owner_card_ids_by_page_id.get(&page.page_id).copied()?,
                    atlas_slot_id,
                    capture_slot_id,
                    atlas_sample_rgba,
                    capture_sample_rgba,
                })
            })
            .collect();
        self.bump_scene_revision_if(self.page_contents != previous_page_contents);
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

    fn rebind_page_contents(&mut self) {
        let existing_entries = self
            .page_contents
            .iter()
            .copied()
            .map(|entry| (entry.page_id, entry))
            .collect::<BTreeMap<_, _>>();
        let atlas_slots_by_page_id = self
            .page_table_entries
            .iter()
            .map(|entry| (entry.page_id, entry.atlas_slot_id))
            .collect::<BTreeMap<_, _>>();
        let capture_slots_by_page_id = self
            .capture_slot_reservations
            .iter()
            .map(|entry| (entry.page_id, entry.capture_slot_id))
            .collect::<BTreeMap<_, _>>();
        let owner_card_ids_by_page_id = self
            .resident_pages
            .iter()
            .map(|entry| (entry.page_id, entry.owner_card_id))
            .collect::<BTreeMap<_, _>>();

        self.page_contents = self
            .resident_pages
            .iter()
            .filter_map(|page| {
                let owner_card_id = owner_card_ids_by_page_id.get(&page.page_id).copied()?;
                let existing = existing_entries
                    .get(&page.page_id)
                    .copied()
                    .filter(|entry| entry.owner_card_id == owner_card_id)?;
                Some(HybridGiSurfaceCachePageContentEntry {
                    page_id: page.page_id,
                    owner_card_id,
                    atlas_slot_id: atlas_slots_by_page_id.get(&page.page_id).copied()?,
                    capture_slot_id: capture_slots_by_page_id.get(&page.page_id).copied()?,
                    atlas_sample_rgba: existing.atlas_sample_rgba,
                    capture_sample_rgba: existing.capture_sample_rgba,
                })
            })
            .collect();
    }

    pub(crate) fn resident_page_count(&self) -> usize {
        self.resident_pages.len()
    }

    #[cfg(test)]
    pub(crate) fn resident_page_ids(&self) -> Vec<u32> {
        self.resident_page_ids_snapshot()
    }

    pub(crate) fn dirty_page_count(&self) -> usize {
        self.dirty_page_ids.len()
    }

    pub(crate) fn dirty_page_ids_snapshot(&self) -> Vec<u32> {
        self.dirty_page_ids.clone()
    }

    #[cfg(test)]
    pub(crate) fn dirty_page_ids(&self) -> Vec<u32> {
        self.dirty_page_ids_snapshot()
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

    pub(crate) fn owner_card_ids_by_page_id_snapshot(&self) -> Vec<(u32, u32)> {
        self.resident_pages
            .iter()
            .map(|entry| (entry.page_id, entry.owner_card_id))
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

    pub(crate) fn page_contents_snapshot(&self) -> Vec<(u32, u32, u32, u32, [u8; 4], [u8; 4])> {
        self.page_contents
            .iter()
            .map(|entry| {
                (
                    entry.page_id,
                    entry.owner_card_id,
                    entry.atlas_slot_id,
                    entry.capture_slot_id,
                    entry.atlas_sample_rgba,
                    entry.capture_sample_rgba,
                )
            })
            .collect()
    }

    pub(crate) fn scene_revision(&self) -> u32 {
        self.scene_revision
    }

    #[cfg(test)]
    pub(crate) fn page_contents(&self) -> Vec<(u32, u32, u32, u32, [u8; 4], [u8; 4])> {
        self.page_contents_snapshot()
    }

    #[cfg(test)]
    pub(crate) fn replace_page_contents_for_test(
        &mut self,
        page_contents: &[(u32, u32, u32, u32, [u8; 4], [u8; 4])],
    ) {
        let previous_page_contents = self.page_contents.clone();
        self.page_contents = page_contents
            .iter()
            .copied()
            .map(
                |(
                    page_id,
                    owner_card_id,
                    atlas_slot_id,
                    capture_slot_id,
                    atlas_sample_rgba,
                    capture_sample_rgba,
                )| HybridGiSurfaceCachePageContentEntry {
                    page_id,
                    owner_card_id,
                    atlas_slot_id,
                    capture_slot_id,
                    atlas_sample_rgba,
                    capture_sample_rgba,
                },
            )
            .collect();
        self.bump_scene_revision_if(self.page_contents != previous_page_contents);
    }

    fn resident_page_ids_snapshot(&self) -> Vec<u32> {
        self.resident_pages
            .iter()
            .map(|entry| entry.page_id)
            .collect()
    }

    fn bump_scene_revision_if(&mut self, changed: bool) {
        if changed {
            self.scene_revision = self.scene_revision.wrapping_add(1);
        }
    }
}

fn allocate_page_id(reusable_page_ids: &mut BTreeSet<u32>, next_page_id: &mut u32) -> u32 {
    if let Some(page_id) = reusable_page_ids.iter().next().copied() {
        reusable_page_ids.remove(&page_id);
        return page_id;
    }

    let page_id = *next_page_id;
    *next_page_id = next_page_id.saturating_add(1);
    page_id
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
