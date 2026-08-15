use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    mem::size_of,
    ops::Range,
};

use crate::text::{visit_hard_lines, HardLine};

pub(crate) const DEFAULT_HARD_LINE_INDEX_CACHE_CAPACITY: usize = 16;
pub(crate) const DEFAULT_HARD_LINE_INDEX_CACHE_MAX_BYTES: usize = 32 * 1024 * 1024;

/// Stable owner identity and revision supplied by a retained text document.
///
/// A key is valid only while its owner advances `revision` for every source change.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub(crate) struct TextDocumentKey {
    owner: u64,
    revision: u64,
}

impl TextDocumentKey {
    pub(crate) const fn new(owner: u64, revision: u64) -> Self {
        Self { owner, revision }
    }

    pub(crate) fn fingerprint(self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct HardLineIndexCacheReport {
    pub(crate) entry_count: usize,
    pub(crate) estimated_bytes: usize,
    pub(crate) hit_count: u64,
    pub(crate) miss_count: u64,
    pub(crate) build_count: u64,
    pub(crate) evicted_count: u64,
    pub(crate) oversized_bypass_count: u64,
    pub(crate) unkeyed_bypass_count: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct HardLineIndexEntry {
    key: TextDocumentKey,
    lines: Vec<HardLine>,
    estimated_bytes: usize,
    last_access: u64,
}

/// Bounded source-offset index shared by distinct viewport layouts of one retained document.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HardLineIndexCache {
    entries: Vec<HardLineIndexEntry>,
    capacity: usize,
    max_bytes: usize,
    estimated_bytes: usize,
    next_access: u64,
    report: HardLineIndexCacheReport,
}

impl Default for HardLineIndexCache {
    fn default() -> Self {
        Self::with_limits(
            DEFAULT_HARD_LINE_INDEX_CACHE_CAPACITY,
            DEFAULT_HARD_LINE_INDEX_CACHE_MAX_BYTES,
        )
    }
}

impl HardLineIndexCache {
    pub(crate) fn with_limits(capacity: usize, max_bytes: usize) -> Self {
        Self {
            entries: Vec::new(),
            capacity: capacity.max(1),
            max_bytes: max_bytes.max(size_of::<HardLine>()),
            estimated_bytes: 0,
            next_access: 0,
            report: HardLineIndexCacheReport::default(),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.entries.clear();
        self.estimated_bytes = 0;
        self.refresh_report_size();
    }

    pub(crate) fn report(&self) -> HardLineIndexCacheReport {
        let mut report = self.report;
        report.entry_count = self.entries.len();
        report.estimated_bytes = self.estimated_bytes;
        report
    }

    pub(crate) fn count_and_window(
        &mut self,
        key: TextDocumentKey,
        text: &str,
        range: Range<usize>,
    ) -> (usize, Vec<HardLine>) {
        let access = self.next_access();
        if let Some(index) = self.entries.iter().position(|entry| entry.key == key) {
            self.report.hit_count = self.report.hit_count.saturating_add(1);
            let entry = &mut self.entries[index];
            entry.last_access = access;
            return (entry.lines.len(), line_window(&entry.lines, range));
        }

        self.report.miss_count = self.report.miss_count.saturating_add(1);
        let mut lines = Vec::new();
        let mut window = Vec::new();
        let mut line_count = 0usize;
        let max_cached_lines = self.max_cached_line_count();
        let mut cacheable = true;
        visit_hard_lines(text, |line| {
            if range.contains(&line_count) {
                window.push(line.clone());
            }
            if cacheable {
                cacheable = reserve_hard_line_slot(&mut lines, max_cached_lines);
                if cacheable {
                    lines.push(line);
                } else {
                    lines.clear();
                }
            }
            line_count = line_count.saturating_add(1);
        });

        if !cacheable || estimated_entry_bytes(&lines) > self.max_bytes {
            self.report.oversized_bypass_count =
                self.report.oversized_bypass_count.saturating_add(1);
            return (line_count, window);
        }

        let estimated_bytes = estimated_entry_bytes(&lines);
        self.evict_for(estimated_bytes);
        self.estimated_bytes = self.estimated_bytes.saturating_add(estimated_bytes);
        self.entries.push(HardLineIndexEntry {
            key,
            lines,
            estimated_bytes,
            last_access: access,
        });
        self.report.build_count = self.report.build_count.saturating_add(1);
        self.refresh_report_size();
        (line_count, window)
    }

    fn evict_for(&mut self, incoming_bytes: usize) {
        while self.entries.len() >= self.capacity
            || self.estimated_bytes.saturating_add(incoming_bytes) > self.max_bytes
        {
            let Some((oldest_index, _)) = self
                .entries
                .iter()
                .enumerate()
                .min_by_key(|(_, entry)| entry.last_access)
            else {
                break;
            };
            let entry = self.entries.swap_remove(oldest_index);
            self.estimated_bytes = self.estimated_bytes.saturating_sub(entry.estimated_bytes);
            self.report.evicted_count = self.report.evicted_count.saturating_add(1);
        }
    }

    fn next_access(&mut self) -> u64 {
        self.next_access = self.next_access.saturating_add(1);
        self.next_access
    }

    fn max_cached_line_count(&self) -> usize {
        self.max_bytes
            .saturating_sub(size_of::<HardLineIndexEntry>())
            / size_of::<HardLine>()
    }

    fn refresh_report_size(&mut self) {
        self.report.entry_count = self.entries.len();
        self.report.estimated_bytes = self.estimated_bytes;
    }

    pub(crate) fn record_unkeyed_bypass(&mut self) {
        self.report.unkeyed_bypass_count = self.report.unkeyed_bypass_count.saturating_add(1);
    }
}

fn reserve_hard_line_slot(lines: &mut Vec<HardLine>, max_lines: usize) -> bool {
    if lines.len() >= max_lines {
        return false;
    }
    if lines.len() == lines.capacity() {
        let current_capacity = lines.capacity();
        let next_capacity = current_capacity.max(1).saturating_mul(2).min(max_lines);
        lines.reserve_exact(next_capacity.saturating_sub(current_capacity));
    }
    true
}

fn line_window(lines: &[HardLine], range: Range<usize>) -> Vec<HardLine> {
    let start = range.start.min(lines.len());
    let end = range.end.min(lines.len());
    (start < end)
        .then(|| lines[start..end].to_vec())
        .unwrap_or_default()
}

fn estimated_entry_bytes(lines: &Vec<HardLine>) -> usize {
    lines
        .capacity()
        .saturating_mul(size_of::<HardLine>())
        .saturating_add(size_of::<HardLineIndexEntry>())
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::{HardLineIndexCache, HardLineIndexEntry, TextDocumentKey};
    use crate::text::HardLine;

    #[test]
    fn hard_line_index_cache_reuses_one_document_for_multiple_viewports() {
        let text = "zero\none\ntwo\nthree";
        let mut cache = HardLineIndexCache::with_limits(2, 4 * 1024);

        let key = TextDocumentKey::new(7, 1);
        let (first_count, first_window) = cache.count_and_window(key, text, 1..2);
        let (second_count, second_window) = cache.count_and_window(key, text, 3..4);

        assert_eq!(first_count, 4);
        assert_eq!(second_count, 4);
        assert_eq!(first_window[0].content, 5..8);
        assert_eq!(second_window[0].content, 13..18);
        assert_eq!(cache.report().build_count, 1);
        assert_eq!(cache.report().hit_count, 1);
    }

    #[test]
    fn hard_line_index_cache_reuses_unicode_crlf_document_windows() {
        let text = "first\r\n世界\u{2028}third";
        let mut cache = HardLineIndexCache::with_limits(2, 4 * 1024);
        let key = TextDocumentKey::new(7, 1);

        let (first_count, first_window) = cache.count_and_window(key, text, 0..1);
        let (second_count, second_window) = cache.count_and_window(key, text, 1..2);

        assert_eq!(first_count, 3);
        assert_eq!(second_count, 3);
        assert_eq!(first_window[0].content, 0..5);
        assert_eq!(second_window[0].content, 7..13);
        assert_eq!(cache.report().build_count, 1);
        assert_eq!(cache.report().hit_count, 1);
    }

    #[test]
    fn hard_line_index_cache_rebuilds_when_document_revision_changes() {
        let mut cache = HardLineIndexCache::with_limits(2, 4 * 1024);

        let (before_count, _) =
            cache.count_and_window(TextDocumentKey::new(7, 1), "zero\none", 0..1);
        let (after_count, after_window) =
            cache.count_and_window(TextDocumentKey::new(7, 2), "zero\none\ntwo", 2..3);

        assert_eq!(before_count, 2);
        assert_eq!(after_count, 3);
        assert_eq!(after_window[0].content, 9..12);
        assert_eq!(cache.report().build_count, 2);
    }

    #[test]
    fn hard_line_index_cache_evicts_the_least_recent_document_at_capacity() {
        let mut cache = HardLineIndexCache::with_limits(2, 4 * 1024);
        let first = TextDocumentKey::new(1, 1);
        let second = TextDocumentKey::new(2, 1);
        let third = TextDocumentKey::new(3, 1);

        cache.count_and_window(first, "first", 0..1);
        cache.count_and_window(second, "second", 0..1);
        cache.count_and_window(first, "first", 0..1);
        cache.count_and_window(third, "third", 0..1);
        let (count, window) = cache.count_and_window(second, "second", 0..1);

        assert_eq!(count, 1);
        assert_eq!(window[0].content, 0..6);
        assert_eq!(cache.report().entry_count, 2);
        assert_eq!(cache.report().build_count, 4);
        assert_eq!(cache.report().hit_count, 1);
        assert_eq!(cache.report().evicted_count, 2);
    }

    #[test]
    fn hard_line_index_cache_evicts_the_least_recent_document_for_the_byte_budget() {
        let entry_bytes = size_of::<HardLineIndexEntry>() + (2 * size_of::<HardLine>());
        let mut cache = HardLineIndexCache::with_limits(8, entry_bytes * 2);
        let first = TextDocumentKey::new(1, 1);
        let second = TextDocumentKey::new(2, 1);
        let third = TextDocumentKey::new(3, 1);

        cache.count_and_window(first, "first", 0..1);
        cache.count_and_window(second, "second", 0..1);
        cache.count_and_window(first, "first", 0..1);
        cache.count_and_window(third, "third", 0..1);
        let (count, window) = cache.count_and_window(second, "second", 0..1);

        assert_eq!(count, 1);
        assert_eq!(window[0].content, 0..6);
        assert_eq!(cache.report().entry_count, 2);
        assert_eq!(cache.report().estimated_bytes, entry_bytes * 2);
        assert_eq!(cache.report().build_count, 4);
        assert_eq!(cache.report().hit_count, 1);
        assert_eq!(cache.report().evicted_count, 2);
    }

    #[test]
    fn oversized_documents_return_only_the_requested_window_without_retaining_an_index() {
        let mut cache = HardLineIndexCache::with_limits(2, size_of::<HardLine>());

        let (line_count, window) =
            cache.count_and_window(TextDocumentKey::new(7, 1), "zero\none\ntwo", 1..2);

        assert_eq!(line_count, 3);
        assert_eq!(window[0].content, 5..8);
        assert_eq!(cache.report().entry_count, 0);
        assert_eq!(cache.report().build_count, 0);
        assert_eq!(cache.report().oversized_bypass_count, 1);
    }
}
