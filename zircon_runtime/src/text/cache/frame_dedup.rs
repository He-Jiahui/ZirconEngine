use std::{hash::Hash, sync::Arc};

use super::index::{IndexedTextCache, IndexedTextCacheEntry, TextCacheSlot};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextFrameDedupReport {
    pub(crate) frame_index: u64,
    pub(crate) entry_count: usize,
    pub(crate) hit_count: u64,
    pub(crate) miss_count: u64,
    pub(crate) collision_miss_count: u64,
    pub(crate) lookup_candidate_count: u64,
    pub(crate) insert_count: u64,
    pub(crate) update_count: u64,
    pub(crate) clear_count: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct TextFrameDedupEntry<K, V> {
    key: K,
    text: Arc<str>,
    value: V,
}

impl<K, V> IndexedTextCacheEntry<K> for TextFrameDedupEntry<K, V> {
    fn cache_key(&self) -> &K {
        &self.key
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextFrameDedup<K: Eq + Hash, V> {
    index: IndexedTextCache<K, TextFrameDedupEntry<K, V>>,
    frame_report: TextFrameDedupReport,
}

impl<K, V> Default for TextFrameDedup<K, V>
where
    K: Clone + Eq + Hash,
{
    fn default() -> Self {
        Self {
            index: IndexedTextCache::new(),
            frame_report: TextFrameDedupReport::default(),
        }
    }
}

impl<K, V> TextFrameDedup<K, V>
where
    K: Clone + Eq + Hash,
{
    pub(crate) fn begin_frame(&mut self, frame_index: u64) {
        self.index.clear();
        self.frame_report = TextFrameDedupReport {
            frame_index,
            ..TextFrameDedupReport::default()
        };
    }

    pub(crate) fn clear(&mut self) {
        self.index.clear();
        self.frame_report.clear_count = self.frame_report.clear_count.saturating_add(1);
        self.refresh_report_size();
    }

    pub(crate) fn len(&self) -> usize {
        self.index.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    pub(crate) fn report(&self) -> TextFrameDedupReport {
        let mut report = self.frame_report;
        report.entry_count = self.index.len();
        report
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
        let (_, entry, inserted) = self.index.update_or_insert_with(
            lookup.slot.filter(|slot| self.index.entry(*slot).is_some()),
            value,
            false,
            |entry, value| entry.value = value,
            |value| TextFrameDedupEntry { key, text, value },
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
        produce: impl FnOnce() -> V,
    ) -> (&V, bool) {
        let text = text.into();
        let slot = self
            .find_slot(&key, text.as_ref())
            .filter(|slot| self.index.entry(*slot).is_some());
        let (_, entry, inserted) = self.index.update_or_insert_with(
            slot,
            (),
            false,
            |_, ()| {},
            |()| TextFrameDedupEntry {
                key,
                text,
                value: produce(),
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
        if let Some(slot) = lookup.slot {
            self.frame_report.hit_count = self.frame_report.hit_count.saturating_add(1);
            return Some(slot);
        }

        self.frame_report.miss_count = self.frame_report.miss_count.saturating_add(1);
        if collision_seen {
            self.frame_report.collision_miss_count =
                self.frame_report.collision_miss_count.saturating_add(1);
        }
        None
    }

    fn record_lookup(&mut self, candidate_count: usize) {
        self.frame_report.lookup_candidate_count = self
            .frame_report
            .lookup_candidate_count
            .saturating_add(candidate_count as u64);
    }

    fn refresh_report_size(&mut self) {
        self.frame_report.entry_count = self.index.len();
    }
}
