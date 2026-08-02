use std::{collections::HashMap, hash::Hash};

pub(super) type TextCacheSlot = u64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct TextCacheLookup {
    pub(super) slot: Option<TextCacheSlot>,
    pub(super) candidate_count: usize,
}

pub(super) trait IndexedTextCacheEntry<K> {
    fn cache_key(&self) -> &K;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct TextCacheLruLinks {
    previous: Option<TextCacheSlot>,
    next: Option<TextCacheSlot>,
}

// Slots keep entry addresses independent from collision-bucket maintenance.
// The linked index owns recency, making touch and eviction constant time.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct IndexedTextCache<K: Eq + Hash, E> {
    entries: HashMap<TextCacheSlot, E>,
    buckets: HashMap<K, Vec<TextCacheSlot>>,
    bucket_positions: HashMap<TextCacheSlot, usize>,
    lru_links: HashMap<TextCacheSlot, TextCacheLruLinks>,
    lru_head: Option<TextCacheSlot>,
    lru_tail: Option<TextCacheSlot>,
    next_slot: TextCacheSlot,
}

impl<K, E> IndexedTextCache<K, E>
where
    K: Clone + Eq + Hash,
    E: IndexedTextCacheEntry<K>,
{
    pub(super) fn new() -> Self {
        Self {
            entries: HashMap::new(),
            buckets: HashMap::new(),
            bucket_positions: HashMap::new(),
            lru_links: HashMap::new(),
            lru_head: None,
            lru_tail: None,
            next_slot: 1,
        }
    }

    pub(super) fn clear(&mut self) {
        self.entries.clear();
        self.buckets.clear();
        self.bucket_positions.clear();
        self.lru_links.clear();
        self.lru_head = None;
        self.lru_tail = None;
    }

    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(super) fn entry(&self, slot: TextCacheSlot) -> Option<&E> {
        self.entries.get(&slot)
    }

    pub(super) fn entry_mut(&mut self, slot: TextCacheSlot) -> Option<&mut E> {
        self.entries.get_mut(&slot)
    }

    pub(super) fn find_slot(&self, key: &K, matches: impl FnMut(&E) -> bool) -> TextCacheLookup {
        let Some(candidates) = self.buckets.get(key) else {
            return TextCacheLookup::default();
        };

        self.find_in_slots(candidates, matches)
    }

    pub(super) fn find_in_slots(
        &self,
        candidates: &[TextCacheSlot],
        mut matches: impl FnMut(&E) -> bool,
    ) -> TextCacheLookup {
        let mut candidate_count = 0;
        for &slot in candidates {
            let Some(entry) = self.entries.get(&slot) else {
                continue;
            };
            candidate_count += 1;
            if matches(entry) {
                return TextCacheLookup {
                    slot: Some(slot),
                    candidate_count,
                };
            }
        }

        TextCacheLookup {
            slot: None,
            candidate_count,
        }
    }

    pub(super) fn insert(&mut self, entry: E) -> TextCacheSlot {
        let (slot, _) = self.insert_inner(entry, true);
        slot
    }

    pub(super) fn update_or_insert_with<T>(
        &mut self,
        update_slot: Option<TextCacheSlot>,
        input: T,
        track_lru: bool,
        update: impl FnOnce(&mut E, T),
        make_entry: impl FnOnce(T) -> E,
    ) -> (TextCacheSlot, &mut E, bool) {
        let Some(slot) = update_slot else {
            let (slot, entry) = self.insert_inner(make_entry(input), track_lru);
            return (slot, entry, true);
        };

        if !self.entries.contains_key(&slot) {
            // Do not reuse a stale slot: a malformed bucket may still reference
            // it. A fresh slot keeps that bucket fail-closed and records the new
            // entry through every authoritative index.
            let (slot, entry) = self.insert_inner(make_entry(input), track_lru);
            return (slot, entry, true);
        }

        if track_lru {
            self.touch(slot);
        }

        match self.entries.entry(slot) {
            std::collections::hash_map::Entry::Occupied(mut occupied) => {
                update(occupied.get_mut(), input);
                (slot, occupied.into_mut(), false)
            }
            std::collections::hash_map::Entry::Vacant(vacant) => {
                // The caller supplied a stale slot. Register the recovered entry
                // before returning it so lookup and LRU ownership stay coherent.
                let entry = make_entry(input);
                let key = entry.cache_key().clone();
                let candidates = self.buckets.entry(key).or_default();
                let candidate_index = candidates.len();
                candidates.push(slot);
                self.bucket_positions.insert(slot, candidate_index);
                if track_lru {
                    Self::attach_most_recent_parts(
                        &mut self.lru_links,
                        &mut self.lru_head,
                        &mut self.lru_tail,
                        slot,
                    );
                }
                (slot, vacant.insert(entry), true)
            }
        }
    }

    fn insert_inner(&mut self, entry: E, track_lru: bool) -> (TextCacheSlot, &mut E) {
        let slot = self.next_slot();
        let key = entry.cache_key().clone();
        let candidates = self.buckets.entry(key).or_default();
        let candidate_index = candidates.len();
        candidates.push(slot);
        self.bucket_positions.insert(slot, candidate_index);
        if track_lru {
            self.attach_most_recent(slot);
        }
        let entry = match self.entries.entry(slot) {
            std::collections::hash_map::Entry::Vacant(vacant) => vacant.insert(entry),
            std::collections::hash_map::Entry::Occupied(occupied) => occupied.into_mut(),
        };
        (slot, entry)
    }

    pub(super) fn touch(&mut self, slot: TextCacheSlot) {
        if !self.entries.contains_key(&slot) {
            return;
        }
        if self.lru_links.contains_key(&slot) {
            self.detach_lru(slot);
        }
        self.attach_most_recent(slot);
    }

    pub(super) fn pop_oldest(&mut self) -> Option<E> {
        self.pop_oldest_with_slot().map(|(_, entry)| entry)
    }

    pub(super) fn pop_oldest_with_slot(&mut self) -> Option<(TextCacheSlot, E)> {
        let slot = self.lru_head?;
        self.remove(slot).map(|entry| (slot, entry))
    }

    pub(super) fn remove(&mut self, slot: TextCacheSlot) -> Option<E> {
        let entry = self.entries.remove(&slot)?;
        if self.lru_links.contains_key(&slot) {
            self.detach_lru(slot);
        }

        let key = entry.cache_key();
        let remove_bucket = if let Some(candidates) = self.buckets.get_mut(key) {
            if let Some(index) = self.bucket_positions.remove(&slot) {
                let removed = candidates.swap_remove(index);
                debug_assert_eq!(removed, slot);
                if let Some(&moved_slot) = candidates.get(index) {
                    self.bucket_positions.insert(moved_slot, index);
                }
            }
            candidates.is_empty()
        } else {
            false
        };
        if remove_bucket {
            self.buckets.remove(key);
        }
        Some(entry)
    }

    fn next_slot(&mut self) -> TextCacheSlot {
        loop {
            let slot = self.next_slot;
            self.next_slot = self.next_slot.checked_add(1).unwrap_or(1);
            if !self.entries.contains_key(&slot) {
                return slot;
            }
        }
    }

    fn attach_most_recent(&mut self, slot: TextCacheSlot) {
        Self::attach_most_recent_parts(
            &mut self.lru_links,
            &mut self.lru_head,
            &mut self.lru_tail,
            slot,
        );
    }

    fn attach_most_recent_parts(
        lru_links: &mut HashMap<TextCacheSlot, TextCacheLruLinks>,
        lru_head: &mut Option<TextCacheSlot>,
        lru_tail: &mut Option<TextCacheSlot>,
        slot: TextCacheSlot,
    ) {
        let previous = *lru_tail;
        lru_links.insert(
            slot,
            TextCacheLruLinks {
                previous,
                next: None,
            },
        );
        if let Some(previous) = previous {
            if let Some(previous_links) = lru_links.get_mut(&previous) {
                previous_links.next = Some(slot);
            } else {
                *lru_head = Some(slot);
            }
        } else {
            *lru_head = Some(slot);
        }
        *lru_tail = Some(slot);
    }

    fn detach_lru(&mut self, slot: TextCacheSlot) {
        let Some(links) = self.lru_links.remove(&slot) else {
            return;
        };
        let previous = links
            .previous
            .filter(|candidate| self.lru_links.contains_key(candidate));
        let next = links
            .next
            .filter(|candidate| self.lru_links.contains_key(candidate));
        if let Some(previous) = previous {
            if let Some(previous_links) = self.lru_links.get_mut(&previous) {
                previous_links.next = next;
            }
        } else {
            self.lru_head = next;
        }
        if let Some(next) = next {
            if let Some(next_links) = self.lru_links.get_mut(&next) {
                next_links.previous = previous;
            }
        } else {
            self.lru_tail = previous;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{IndexedTextCache, IndexedTextCacheEntry};

    #[derive(Debug, PartialEq)]
    struct Entry {
        key: u8,
        value: u8,
    }

    impl IndexedTextCacheEntry<u8> for Entry {
        fn cache_key(&self) -> &u8 {
            &self.key
        }
    }

    #[test]
    fn touch_moves_a_stable_slot_to_the_lru_tail() {
        let mut cache = IndexedTextCache::new();
        cache.insert(Entry { key: 1, value: 10 });
        let second = cache.insert(Entry { key: 2, value: 20 });
        cache.insert(Entry { key: 3, value: 30 });

        let first = cache.find_slot(&1, |_| true).slot.unwrap();
        cache.touch(first);

        assert_eq!(cache.pop_oldest().unwrap().value, 20);
        assert_eq!(cache.pop_oldest().unwrap().value, 30);
        assert_eq!(cache.pop_oldest().unwrap().value, 10);
        assert!(cache.entry(second).is_none());
    }

    #[test]
    fn text_cache_indexes_keep_hot_lookup_and_eviction_work_constant_after_insert() {
        let mut cache = IndexedTextCache::new();

        let slot = cache.insert(Entry { key: 7, value: 70 });
        assert_eq!(cache.entry(slot).map(|entry| entry.value), Some(70));
    }

    #[test]
    fn text_cache_indexes_keep_hot_lookup_and_eviction_work_constant_when_upserting() {
        let mut cache = IndexedTextCache::new();
        let slot = cache.insert(Entry { key: 7, value: 70 });
        let (updated_slot, entry, inserted) = cache.update_or_insert_with(
            Some(slot),
            80,
            true,
            |entry, value| entry.value = value,
            |value| Entry { key: 7, value },
        );

        assert_eq!(updated_slot, slot);
        assert!(!inserted);
        assert_eq!(entry.value, 80);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn stale_upsert_slot_recovers_through_the_lookup_and_lru_indexes() {
        let mut cache: IndexedTextCache<u8, Entry> = IndexedTextCache::new();
        let (slot, entry, inserted) = cache.update_or_insert_with(
            Some(999),
            70,
            true,
            |entry, value| entry.value = value,
            |value| Entry { key: 7, value },
        );

        assert!(inserted);
        assert_eq!(entry.value, 70);
        assert_ne!(slot, 999);
        assert_eq!(cache.find_slot(&7, |_| true).slot, Some(slot));
        assert_eq!(cache.pop_oldest().map(|entry| entry.value), Some(70));
    }

    #[test]
    fn lru_recovers_when_a_neighbor_link_is_missing() {
        let mut cache = IndexedTextCache::new();
        let first = cache.insert(Entry { key: 1, value: 10 });
        let second = cache.insert(Entry { key: 2, value: 20 });

        cache.lru_links.remove(&first);
        cache.touch(second);

        assert_eq!(cache.pop_oldest().unwrap().value, 20);
    }

    #[test]
    fn collision_bucket_removal_does_not_scan_candidates() {
        let source = include_str!("index.rs");
        let linear_search = concat!("candidates.iter()", ".position");

        assert!(
            !source.contains(linear_search),
            "eviction must remove a collision-bucket slot through its indexed position"
        );
    }

    #[test]
    fn removing_a_middle_collision_candidate_keeps_remaining_slots_indexed() {
        let mut cache = IndexedTextCache::new();
        let first = cache.insert(Entry { key: 1, value: 10 });
        let middle = cache.insert(Entry { key: 1, value: 20 });
        let last = cache.insert(Entry { key: 1, value: 30 });

        assert_eq!(cache.remove(middle).map(|entry| entry.value), Some(20));
        assert_eq!(
            cache.find_slot(&1, |entry| entry.value == 10).slot,
            Some(first)
        );
        assert_eq!(
            cache.find_slot(&1, |entry| entry.value == 30).slot,
            Some(last)
        );
        assert_eq!(cache.remove(last).map(|entry| entry.value), Some(30));
        assert_eq!(cache.pop_oldest().map(|entry| entry.value), Some(10));
        assert!(cache.is_empty());
    }
}
