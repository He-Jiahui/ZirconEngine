use std::{hash::Hash, sync::Arc};

use super::index::{IndexedTextCache, IndexedTextCacheEntry, TextCacheSlot};

pub(crate) const DEFAULT_TEXT_MEASURE_CACHE_CAPACITY: usize = 4096;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextMeasureCacheReport {
    pub(crate) frame_index: u64,
    pub(crate) capacity: usize,
    pub(crate) entry_count: usize,
    pub(crate) hit_count: u64,
    pub(crate) miss_count: u64,
    pub(crate) collision_miss_count: u64,
    pub(crate) lookup_candidate_count: u64,
    pub(crate) eviction_scan_count: u64,
    pub(crate) entry_move_count: u64,
    pub(crate) insert_count: u64,
    pub(crate) update_count: u64,
    pub(crate) evicted_count: u64,
    pub(crate) trim_count: u64,
    pub(crate) clear_count: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct TextMeasureCacheEntry<K, V> {
    key: K,
    text: Arc<str>,
    value: V,
}

impl<K, V> IndexedTextCacheEntry<K> for TextMeasureCacheEntry<K, V> {
    fn cache_key(&self) -> &K {
        &self.key
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextMeasureCache<K: Eq + Hash, V> {
    index: IndexedTextCache<K, TextMeasureCacheEntry<K, V>>,
    capacity: usize,
    frame_report: TextMeasureCacheReport,
}

impl<K, V> Default for TextMeasureCache<K, V>
where
    K: Clone + Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> TextMeasureCache<K, V>
where
    K: Clone + Eq + Hash,
{
    pub(crate) fn new() -> Self {
        Self::with_capacity(DEFAULT_TEXT_MEASURE_CACHE_CAPACITY)
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        let mut cache = Self {
            index: IndexedTextCache::new(),
            capacity,
            frame_report: TextMeasureCacheReport::default(),
        };
        cache.frame_report.capacity = capacity;
        cache
    }

    pub(crate) fn begin_frame(&mut self, frame_index: u64) {
        self.frame_report = TextMeasureCacheReport {
            frame_index,
            capacity: self.capacity,
            entry_count: self.index.len(),
            ..TextMeasureCacheReport::default()
        };
    }

    pub(crate) fn finish_frame(&mut self) {
        self.trim_to_capacity();
    }

    pub(crate) fn clear(&mut self) {
        self.frame_report.evicted_count = self
            .frame_report
            .evicted_count
            .saturating_add(self.index.len() as u64);
        self.frame_report.clear_count = self.frame_report.clear_count.saturating_add(1);
        self.index.clear();
        self.refresh_report_size();
    }

    pub(crate) fn len(&self) -> usize {
        self.index.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    pub(crate) fn report(&self) -> TextMeasureCacheReport {
        let mut report = self.frame_report;
        report.entry_count = self.index.len();
        report
    }

    pub(crate) fn contains_exact(&self, key: &K, text: &str) -> bool {
        self.index
            .find_slot(key, |entry| entry.text.as_ref() == text)
            .slot
            .is_some()
    }

    pub(crate) fn get(&mut self, key: &K, text: &str) -> Option<&V> {
        let slot = self.find_slot(key, text)?;
        self.index.entry(slot).map(|entry| &entry.value)
    }

    pub(crate) fn insert(&mut self, key: K, text: impl Into<Arc<str>>, value: V) -> &V {
        let text = text.into();
        let lookup = self
            .index
            .find_slot(&key, |entry| entry.text.as_ref() == text.as_ref());
        self.record_lookup(lookup.candidate_count);
        let slot = lookup.slot.filter(|slot| self.index.entry(*slot).is_some());
        if slot.is_none() {
            self.trim_before_insert();
        }
        let (_, entry, inserted) = self.index.update_or_insert_with(
            slot,
            value,
            true,
            |entry, value| entry.value = value,
            |value| TextMeasureCacheEntry { key, text, value },
        );
        if inserted {
            self.frame_report.insert_count = self.frame_report.insert_count.saturating_add(1);
        } else {
            self.frame_report.update_count = self.frame_report.update_count.saturating_add(1);
        }
        &entry.value
    }

    pub(crate) fn get_or_insert_with(
        &mut self,
        key: K,
        text: impl Into<Arc<str>>,
        measure: impl FnOnce() -> V,
    ) -> (&V, bool) {
        let text = text.into();
        let slot = self
            .find_slot(&key, text.as_ref())
            .filter(|slot| self.index.entry(*slot).is_some());
        if slot.is_none() {
            self.trim_before_insert();
        }
        let (_, entry, inserted) = self.index.update_or_insert_with(
            slot,
            (),
            true,
            |_, ()| {},
            |()| TextMeasureCacheEntry {
                key,
                text,
                value: measure(),
            },
        );
        if inserted {
            self.frame_report.insert_count = self.frame_report.insert_count.saturating_add(1);
        }
        (&entry.value, inserted)
    }

    fn find_slot(&mut self, key: &K, text: &str) -> Option<TextCacheSlot> {
        let mut collision_seen = false;
        let lookup = self.index.find_slot(key, |entry| {
            if entry.text.as_ref() == text {
                true
            } else {
                collision_seen = true;
                false
            }
        });
        self.record_lookup(lookup.candidate_count);
        let Some(slot) = lookup.slot else {
            self.frame_report.miss_count = self.frame_report.miss_count.saturating_add(1);
            if collision_seen {
                self.frame_report.collision_miss_count =
                    self.frame_report.collision_miss_count.saturating_add(1);
            }
            return None;
        };

        self.index.touch(slot);
        self.frame_report.hit_count = self.frame_report.hit_count.saturating_add(1);
        Some(slot)
    }

    fn trim_before_insert(&mut self) {
        let mut evicted = 0_u64;
        while self.index.len() >= self.capacity {
            if self.index.pop_oldest().is_none() {
                break;
            }
            evicted = evicted.saturating_add(1);
        }
        self.record_evictions(evicted);
    }

    fn trim_to_capacity(&mut self) {
        let mut evicted = 0_u64;
        while self.index.len() > self.capacity {
            if self.index.pop_oldest().is_none() {
                break;
            }
            evicted = evicted.saturating_add(1);
        }
        self.record_evictions(evicted);
    }

    fn record_lookup(&mut self, candidate_count: usize) {
        self.frame_report.lookup_candidate_count = self
            .frame_report
            .lookup_candidate_count
            .saturating_add(candidate_count as u64);
    }

    fn record_evictions(&mut self, evicted: u64) {
        if evicted > 0 {
            self.frame_report.evicted_count =
                self.frame_report.evicted_count.saturating_add(evicted);
            self.frame_report.trim_count = self.frame_report.trim_count.saturating_add(1);
        }
        self.refresh_report_size();
    }

    fn refresh_report_size(&mut self) {
        self.frame_report.entry_count = self.index.len();
    }
}
