use std::collections::{BTreeSet, HashMap, hash_map::Entry};
use std::hash::{BuildHasherDefault, Hasher};
#[cfg(test)]
use std::mem::size_of;
use std::num::NonZeroU64;

const SPARSE_LOCATOR_PAGE_BITS: u32 = 8;
pub(super) const SPARSE_LOCATOR_PAGE_SLOTS: usize = 1 << SPARSE_LOCATOR_PAGE_BITS;
const SPARSE_LOCATOR_PAGE_MASK: u32 = SPARSE_LOCATOR_PAGE_SLOTS as u32 - 1;
// Hysteresis bounds retained prefix slots while avoiding representation churn.
const SPARSE_LOCATOR_PREFIX_PROMOTION_FACTOR: usize = 1_024;
const SPARSE_LOCATOR_PREFIX_DEMOTION_FACTOR: usize = 2_048;
const SPARSE_LOCATOR_DIRECTORY_SLACK_FACTOR: usize = 2;
const SPARSE_LOCATOR_MIN_RETAINED_PAGE_REFERENCES: usize = 16;

#[derive(Clone, Copy)]
pub(super) struct SparseRowLocation(NonZeroU64);

impl SparseRowLocation {
    pub(super) fn new(generation: u32, dense_row: usize) -> Self {
        let dense_row = u32::try_from(dense_row)
            .expect("sparse dense row must fit the entity allocator domain");
        let stored_row = dense_row
            .checked_add(1)
            .expect("the invalid entity index keeps one-based dense rows representable");
        let encoded = (u64::from(generation) << 32) | u64::from(stored_row);
        Self(
            NonZeroU64::new(encoded)
                .expect("a one-based dense row keeps the sparse location non-zero"),
        )
    }

    #[inline(always)]
    pub(super) fn generation(self) -> u32 {
        (self.0.get() >> 32) as u32
    }

    #[inline(always)]
    pub(super) fn dense_row(self) -> usize {
        (self.0.get() as u32 - 1) as usize
    }
}

struct SparseLocatorPage {
    rows: [Option<SparseRowLocation>; SPARSE_LOCATOR_PAGE_SLOTS],
    occupied_rows: u16,
}

impl Default for SparseLocatorPage {
    fn default() -> Self {
        Self {
            rows: [None; SPARSE_LOCATOR_PAGE_SLOTS],
            occupied_rows: 0,
        }
    }
}

impl SparseLocatorPage {
    #[inline(always)]
    fn get(&self, slot: usize) -> Option<SparseRowLocation> {
        self.rows[slot]
    }

    fn insert(&mut self, slot: usize, location: SparseRowLocation) -> Option<SparseRowLocation> {
        let previous = self.rows[slot].replace(location);
        if previous.is_none() {
            self.occupied_rows += 1;
        }
        previous
    }

    fn remove(&mut self, slot: usize) -> Option<SparseRowLocation> {
        let removed = self.rows[slot].take()?;
        self.occupied_rows -= 1;
        Some(removed)
    }

    fn is_empty(&self) -> bool {
        self.occupied_rows == 0
    }
}

// Keys are private u32 page indices derived from EntityRegistry-issued handles.
// Do not reuse this hasher at a boundary that accepts caller-controlled keys.
#[derive(Default)]
struct SparseLocatorIdentityHasher(u64);

impl Hasher for SparseLocatorIdentityHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut value = 0_u64;
        for (shift, byte) in bytes.iter().copied().take(8).enumerate() {
            value |= u64::from(byte) << (shift * 8);
        }
        self.0 = value;
    }

    fn write_u32(&mut self, value: u32) {
        self.0 = u64::from(value);
    }
}

type SparseLocatorBuildHasher = BuildHasherDefault<SparseLocatorIdentityHasher>;

#[derive(Default)]
pub(super) struct SparseRowLocator {
    flat_prefix: Vec<Option<SparseRowLocation>>,
    flat_location_count: usize,
    flat_window_base: u32,
    flat_window: Vec<Option<SparseRowLocation>>,
    flat_window_location_count: usize,
    sparse_pages: HashMap<u32, Box<SparseLocatorPage>, SparseLocatorBuildHasher>,
    sparse_page_keys: BTreeSet<u32>,
    location_count: usize,
}

impl SparseRowLocator {
    #[inline(always)]
    pub(super) fn get(&self, index: u32) -> Option<SparseRowLocation> {
        if index >= self.flat_window_base {
            let window_slot = (index - self.flat_window_base) as usize;
            if window_slot < self.flat_window.len() {
                return self.flat_window[window_slot];
            }
        }
        if (index as usize) < self.flat_prefix.len() {
            self.flat_prefix[index as usize]
        } else {
            let (page_index, slot) = split_locator_index(index);
            self.sparse_pages.get(&page_index)?.get(slot)
        }
    }

    pub(super) fn insert(
        &mut self,
        index: u32,
        location: SparseRowLocation,
    ) -> Option<SparseRowLocation> {
        let previous = if (index as usize) < self.flat_prefix.len() {
            let previous = self.flat_prefix[index as usize].replace(location);
            if previous.is_none() {
                self.flat_location_count += 1;
            }
            previous
        } else if let Some(slot) = self.flat_window_slot(index) {
            let previous = self.flat_window[slot].replace(location);
            if previous.is_none() {
                self.flat_window_location_count += 1;
            }
            previous
        } else {
            self.insert_sparse(index, location)
        };
        if previous.is_none() {
            self.location_count += 1;
            self.promote_qualified_prefix();
            self.promote_qualified_window(index);
        }
        previous
    }

    pub(super) fn remove(&mut self, index: u32) -> Option<SparseRowLocation> {
        let removed = if (index as usize) < self.flat_prefix.len() {
            let removed = self.flat_prefix[index as usize].take()?;
            self.flat_location_count -= 1;
            removed
        } else if let Some(slot) = self.flat_window_slot(index) {
            let removed = self.flat_window[slot].take()?;
            self.flat_window_location_count -= 1;
            removed
        } else {
            self.remove_sparse(index)?
        };

        self.location_count -= 1;
        if self.location_count == 0 {
            self.flat_prefix = Vec::new();
            self.flat_location_count = 0;
            self.flat_window_base = 0;
            self.flat_window = Vec::new();
            self.flat_window_location_count = 0;
            self.sparse_pages = HashMap::default();
            self.sparse_page_keys = BTreeSet::new();
        } else {
            // Promotion may borrow global density from another representation. Recheck both spans
            // after every removal so deleting that support cannot retain an under-dense span.
            self.compact_flat_window();
            self.compact_flat_prefix();
            self.compact_flat_window();
        }
        Some(removed)
    }

    fn insert_sparse(
        &mut self,
        index: u32,
        location: SparseRowLocation,
    ) -> Option<SparseRowLocation> {
        let (page_index, slot) = split_locator_index(index);
        let page = match self.sparse_pages.entry(page_index) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                self.sparse_page_keys.insert(page_index);
                entry.insert(Box::new(SparseLocatorPage::default()))
            }
        };
        page.insert(slot, location)
    }

    fn remove_sparse(&mut self, index: u32) -> Option<SparseRowLocation> {
        let (page_index, slot) = split_locator_index(index);
        let page = self.sparse_pages.get_mut(&page_index)?;
        let removed = page.remove(slot)?;
        if page.is_empty() {
            self.sparse_pages.remove(&page_index);
            self.sparse_page_keys.remove(&page_index);
            self.compact_sparse_directory();
        }
        Some(removed)
    }

    fn promote_qualified_prefix(&mut self) {
        let promotable_page = self
            .location_count
            .saturating_mul(SPARSE_LOCATOR_PREFIX_PROMOTION_FACTOR)
            .checked_div(SPARSE_LOCATOR_PAGE_SLOTS)
            .and_then(|page_count| page_count.checked_sub(1))
            .and_then(|page_index| u32::try_from(page_index).ok())
            .unwrap_or(u32::MAX);
        let window_pages = self.flat_window_page_range();
        let absorb_window = window_pages
            .as_ref()
            .is_some_and(|range| *range.end() <= promotable_page);
        let sparse_limit = match window_pages.as_ref() {
            Some(range) if *range.start() <= promotable_page && !absorb_window => {
                range.start().saturating_sub(1)
            }
            _ => promotable_page,
        };
        let pages = self
            .sparse_page_keys
            .range(..=sparse_limit)
            .copied()
            .collect::<Vec<_>>();
        let highest_page = pages
            .last()
            .copied()
            .into_iter()
            .chain(absorb_window.then(|| {
                *window_pages
                    .as_ref()
                    .expect("absorbed window has pages")
                    .end()
            }))
            .max();
        let Some(highest_page) = highest_page else {
            return;
        };

        self.flat_prefix.resize(
            (highest_page as usize + 1) * SPARSE_LOCATOR_PAGE_SLOTS,
            None,
        );
        for page_index in pages {
            self.sparse_page_keys.remove(&page_index);
            let page = self
                .sparse_pages
                .remove(&page_index)
                .expect("queued sparse page remains owned until prefix promotion");
            self.flat_location_count += usize::from(page.occupied_rows);
            let start = page_index as usize * SPARSE_LOCATOR_PAGE_SLOTS;
            self.flat_prefix[start..start + SPARSE_LOCATOR_PAGE_SLOTS].copy_from_slice(&page.rows);
        }
        if absorb_window {
            let start = self.flat_window_base as usize;
            self.flat_prefix[start..start + self.flat_window.len()]
                .copy_from_slice(&self.flat_window);
            self.flat_location_count += self.flat_window_location_count;
            self.clear_flat_window();
        }
        self.compact_sparse_owners();
    }

    fn promote_qualified_window(&mut self, index: u32) {
        let (inserted_page, _) = split_locator_index(index);
        if !self.sparse_page_keys.contains(&inserted_page) {
            return;
        }

        let candidate_base_page = self
            .flat_window_page_range()
            .map_or(inserted_page, |range| (*range.start()).min(inserted_page));
        let candidate_end_page = self
            .flat_window_page_range()
            .map_or(inserted_page, |range| (*range.end()).max(inserted_page));
        let candidate_slots = (candidate_end_page as usize - candidate_base_page as usize + 1)
            * SPARSE_LOCATOR_PAGE_SLOTS;
        if candidate_slots
            > self
                .location_count
                .saturating_mul(SPARSE_LOCATOR_PREFIX_PROMOTION_FACTOR)
        {
            return;
        }

        let candidate_base = candidate_base_page << SPARSE_LOCATOR_PAGE_BITS;
        if self.flat_window.is_empty() {
            self.flat_window_base = candidate_base;
            self.flat_window.resize(candidate_slots, None);
        } else if candidate_base < self.flat_window_base {
            let leading_slots = (self.flat_window_base - candidate_base) as usize;
            let mut expanded = vec![None; candidate_slots];
            expanded[leading_slots..leading_slots + self.flat_window.len()]
                .copy_from_slice(&self.flat_window);
            self.flat_window = expanded;
            self.flat_window_base = candidate_base;
        } else {
            self.flat_window.resize(candidate_slots, None);
        }

        let pages = self
            .sparse_page_keys
            .range(candidate_base_page..=candidate_end_page)
            .copied()
            .collect::<Vec<_>>();
        for page_index in pages {
            self.sparse_page_keys.remove(&page_index);
            let page = self
                .sparse_pages
                .remove(&page_index)
                .expect("indexed sparse page remains owned until window promotion");
            self.flat_window_location_count += usize::from(page.occupied_rows);
            let page_base = page_index << SPARSE_LOCATOR_PAGE_BITS;
            let start = (page_base - self.flat_window_base) as usize;
            self.flat_window[start..start + SPARSE_LOCATOR_PAGE_SLOTS].copy_from_slice(&page.rows);
        }
        self.compact_sparse_owners();
    }

    fn compact_flat_prefix(&mut self) {
        while self.flat_prefix.len() >= SPARSE_LOCATOR_PAGE_SLOTS
            && self.flat_prefix[self.flat_prefix.len() - SPARSE_LOCATOR_PAGE_SLOTS..]
                .iter()
                .all(Option::is_none)
        {
            self.flat_prefix
                .truncate(self.flat_prefix.len() - SPARSE_LOCATOR_PAGE_SLOTS);
        }

        if !self.flat_prefix.is_empty()
            && self
                .flat_location_count
                .saturating_mul(SPARSE_LOCATOR_PREFIX_DEMOTION_FACTOR)
                < self.flat_prefix.len()
        {
            if self.try_rebase_flat_prefix_as_window() {
                return;
            }
            self.demote_flat_prefix();
            return;
        }

        let retained_capacity = self
            .flat_prefix
            .len()
            .saturating_mul(SPARSE_LOCATOR_DIRECTORY_SLACK_FACTOR);
        if self.flat_prefix.capacity() > retained_capacity {
            self.flat_prefix.shrink_to(retained_capacity);
        }
    }

    fn compact_flat_window(&mut self) {
        while self.flat_window.len() >= SPARSE_LOCATOR_PAGE_SLOTS
            && self.flat_window[self.flat_window.len() - SPARSE_LOCATOR_PAGE_SLOTS..]
                .iter()
                .all(Option::is_none)
        {
            self.flat_window
                .truncate(self.flat_window.len() - SPARSE_LOCATOR_PAGE_SLOTS);
        }
        if self.flat_window.is_empty() {
            self.clear_flat_window();
            return;
        }

        if self
            .flat_window_location_count
            .saturating_mul(SPARSE_LOCATOR_PREFIX_DEMOTION_FACTOR)
            < self.flat_window.len()
        {
            self.trim_flat_window_leading_pages();
            if self
                .flat_window_location_count
                .saturating_mul(SPARSE_LOCATOR_PREFIX_DEMOTION_FACTOR)
                < self.flat_window.len()
            {
                self.demote_flat_window();
                return;
            }
        }

        let retained_capacity = self
            .flat_window
            .len()
            .saturating_mul(SPARSE_LOCATOR_DIRECTORY_SLACK_FACTOR);
        if self.flat_window.capacity() > retained_capacity {
            self.flat_window.shrink_to(retained_capacity);
        }
    }

    fn demote_flat_prefix(&mut self) {
        let prefix = std::mem::take(&mut self.flat_prefix);
        self.flat_location_count = 0;
        for (index, location) in prefix.into_iter().enumerate() {
            if let Some(location) = location {
                let previous = self.insert_sparse(index as u32, location);
                debug_assert!(previous.is_none());
            }
        }
    }

    fn try_rebase_flat_prefix_as_window(&mut self) -> bool {
        if !self.flat_window.is_empty() {
            return false;
        }
        let Some(first_location) = self.flat_prefix.iter().position(Option::is_some) else {
            return false;
        };
        let window_base = first_location / SPARSE_LOCATOR_PAGE_SLOTS * SPARSE_LOCATOR_PAGE_SLOTS;
        let window_slots = self.flat_prefix.len() - window_base;
        if self
            .flat_location_count
            .saturating_mul(SPARSE_LOCATOR_PREFIX_PROMOTION_FACTOR)
            < window_slots
        {
            return false;
        }

        self.flat_window = self.flat_prefix.split_off(window_base);
        self.flat_prefix = Vec::new();
        self.flat_window_base = window_base as u32;
        self.flat_window_location_count = self.flat_location_count;
        self.flat_location_count = 0;
        true
    }

    fn trim_flat_window_leading_pages(&mut self) {
        let empty_pages = self
            .flat_window
            .chunks_exact(SPARSE_LOCATOR_PAGE_SLOTS)
            .take_while(|page| page.iter().all(Option::is_none))
            .count();
        if empty_pages == 0 {
            return;
        }
        let empty_slots = empty_pages * SPARSE_LOCATOR_PAGE_SLOTS;
        self.flat_window.drain(..empty_slots);
        self.flat_window_base += empty_slots as u32;
    }

    fn demote_flat_window(&mut self) {
        let base = self.flat_window_base;
        let window = std::mem::take(&mut self.flat_window);
        self.flat_window_base = 0;
        self.flat_window_location_count = 0;
        for (offset, location) in window.into_iter().enumerate() {
            if let Some(location) = location {
                let index = base + offset as u32;
                let previous = self.insert_sparse(index, location);
                debug_assert!(previous.is_none());
            }
        }
    }

    fn compact_sparse_directory(&mut self) {
        let retained_capacity = self
            .sparse_pages
            .len()
            .saturating_mul(SPARSE_LOCATOR_DIRECTORY_SLACK_FACTOR)
            .max(SPARSE_LOCATOR_MIN_RETAINED_PAGE_REFERENCES);
        if self.sparse_pages.capacity() > retained_capacity {
            self.sparse_pages.shrink_to(retained_capacity);
        }
    }

    fn compact_sparse_owners(&mut self) {
        if self.sparse_pages.is_empty() {
            self.sparse_pages = HashMap::default();
            self.sparse_page_keys = BTreeSet::new();
        } else {
            self.compact_sparse_directory();
        }
    }

    #[inline(always)]
    fn flat_window_slot(&self, index: u32) -> Option<usize> {
        if self.flat_window.is_empty() {
            return None;
        }
        let slot = index.checked_sub(self.flat_window_base)? as usize;
        (slot < self.flat_window.len()).then_some(slot)
    }

    fn flat_window_page_range(&self) -> Option<std::ops::RangeInclusive<u32>> {
        if self.flat_window.is_empty() {
            return None;
        }
        let start = self.flat_window_base >> SPARSE_LOCATOR_PAGE_BITS;
        let page_count = self.flat_window.len() / SPARSE_LOCATOR_PAGE_SLOTS;
        Some(start..=start + page_count as u32 - 1)
    }

    fn clear_flat_window(&mut self) {
        self.flat_window_base = 0;
        self.flat_window = Vec::new();
        self.flat_window_location_count = 0;
    }

    #[cfg(test)]
    pub(super) fn page_count(&self) -> usize {
        self.flat_prefix.len() / SPARSE_LOCATOR_PAGE_SLOTS
            + self.flat_window.len() / SPARSE_LOCATOR_PAGE_SLOTS
            + self.sparse_pages.len()
    }

    #[cfg(test)]
    pub(super) fn flat_prefix_slots(&self) -> usize {
        self.flat_prefix.len()
    }

    #[cfg(test)]
    pub(super) fn flat_location_count(&self) -> usize {
        self.flat_location_count
    }

    #[cfg(test)]
    pub(super) fn flat_window_base(&self) -> u32 {
        self.flat_window_base
    }

    #[cfg(test)]
    pub(super) fn flat_window_slots(&self) -> usize {
        self.flat_window.len()
    }

    #[cfg(test)]
    pub(super) fn sparse_page_count(&self) -> usize {
        self.sparse_pages.len()
    }

    #[cfg(test)]
    pub(super) fn sparse_directory_capacity(&self) -> usize {
        self.sparse_pages.capacity() + self.sparse_page_keys.len()
    }

    #[cfg(test)]
    pub(super) fn allocated_bytes(&self) -> usize {
        let prefix_bytes = self.flat_prefix.capacity() * size_of::<Option<SparseRowLocation>>();
        let window_bytes = self.flat_window.capacity() * size_of::<Option<SparseRowLocation>>();
        let directory_bytes = self.sparse_pages.capacity()
            * (size_of::<u32>() + size_of::<Box<SparseLocatorPage>>() + size_of::<u8>());
        let page_bytes = self.sparse_pages.len() * size_of::<SparseLocatorPage>();
        let key_bytes = self.sparse_page_keys.len() * (size_of::<u32>() + size_of::<usize>() * 4);
        prefix_bytes + window_bytes + directory_bytes + page_bytes + key_bytes
    }
}

#[inline(always)]
fn split_locator_index(index: u32) -> (u32, usize) {
    (
        index >> SPARSE_LOCATOR_PAGE_BITS,
        (index & SPARSE_LOCATOR_PAGE_MASK) as usize,
    )
}
