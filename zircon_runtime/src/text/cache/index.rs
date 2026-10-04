//! 测量、布局与帧内去重共用的索引：稳定槽位承载碰撞桶，只有持久缓存把槽位接入淘汰链。
use std::{collections::HashMap, hash::Hash};

pub(super) type TextCacheSlot = u64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct TextCacheLookup {
    pub(super) slot: Option<TextCacheSlot>,
    pub(super) candidate_count: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct TextCacheEvictionWork {
    pub(super) scan_count: usize,
    pub(super) entry_move_count: usize,
}

/// 驻留期间 cache_key 必须保持不变；可变访问及更新回调只能修改结果或统计，不能让桶与条目身份脱节。
pub(crate) trait IndexedTextCacheEntry<K> {
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
pub(crate) struct IndexedTextCache<K: Eq + Hash, E> {
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

    /// 区分只参与查找的条目与持久驻留条目；若要参加淘汰，owner 须显式 touch。
    pub(super) fn insert_untracked(&mut self, entry: E) -> TextCacheSlot {
        let (slot, _) = self.insert_inner(entry, false);
        slot
    }

    /// 调用方先按完整文本和必要的布局条件筛出槽位；这里只维护该槽位的桶与淘汰索引。
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
        self.detach_lru(slot);
        self.attach_most_recent(slot);
    }

    pub(super) fn pop_oldest(&mut self) -> Option<E> {
        self.pop_oldest_with_work().map(|(entry, _)| entry)
    }

    pub(super) fn pop_oldest_with_slot(&mut self) -> Option<(TextCacheSlot, E)> {
        self.pop_oldest_with_slot_and_work()
            .map(|(slot, entry, _)| (slot, entry))
    }

    pub(super) fn pop_oldest_with_work(&mut self) -> Option<(E, TextCacheEvictionWork)> {
        self.pop_oldest_with_slot_and_work()
            .map(|(_, entry, work)| (entry, work))
    }

    pub(super) fn pop_oldest_with_slot_and_work(
        &mut self,
    ) -> Option<(TextCacheSlot, E, TextCacheEvictionWork)> {
        let slot = self.lru_head?;
        self.remove(slot).map(|entry| {
            (
                slot,
                entry,
                TextCacheEvictionWork {
                    // The linked LRU head identifies the victim directly. No resident-entry
                    // search or stable-entry relocation is hidden behind the cache report.
                    scan_count: 1,
                    entry_move_count: 0,
                },
            )
        })
    }

    pub(super) fn remove(&mut self, slot: TextCacheSlot) -> Option<E> {
        let entry = self.entries.remove(&slot)?;
        self.detach_lru(slot);

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
        let mut previous = links.previous;
        if let Some(candidate) = previous {
            if let Some(previous_links) = self.lru_links.get_mut(&candidate) {
                previous_links.next = links.next;
            } else {
                previous = None;
                self.lru_head = links.next;
            }
        } else {
            self.lru_head = links.next;
        }

        let next = if let Some(candidate) = links.next {
            if let Some(next_links) = self.lru_links.get_mut(&candidate) {
                next_links.previous = previous;
                Some(candidate)
            } else {
                if let Some(previous) = previous {
                    if let Some(previous_links) = self.lru_links.get_mut(&previous) {
                        previous_links.next = None;
                    }
                } else {
                    self.lru_head = None;
                }
                None
            }
        } else {
            None
        };
        if next.is_none() {
            self.lru_tail = previous;
        }
    }
}

#[cfg(test)]
#[path = "tests/index.rs"]
mod tests;

#[cfg(test)]
#[path = "index/tests/detach_single_lookup_tests.rs"]
mod detach_single_lookup_tests;
