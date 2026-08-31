use std::{mem::size_of, ops::Range, sync::Arc};

use crate::text::{EphemeralCacheHash, HardLine, visit_hard_lines};

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

    pub(crate) const fn owner(self) -> u64 {
        self.owner
    }

    pub(crate) const fn revision(self) -> u64 {
        self.revision
    }

    pub(crate) fn ephemeral_hash(self) -> EphemeralCacheHash {
        EphemeralCacheHash::from_hashable(&self)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct HardLineIndexCacheBudgetSnapshot {
    pub(crate) max_entries: usize,
    pub(crate) max_bytes: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct HardLineIndexCacheReport {
    pub(crate) budget: HardLineIndexCacheBudgetSnapshot,
    pub(crate) entry_count: usize,
    pub(crate) estimated_bytes: usize,
    pub(crate) hit_count: u64,
    pub(crate) miss_count: u64,
    pub(crate) build_count: u64,
    pub(crate) source_pointer_hit_count: u64,
    pub(crate) source_exact_compare_count: u64,
    pub(crate) stale_source_alias_count: u64,
    pub(crate) evicted_count: u64,
    pub(crate) oversized_bypass_count: u64,
    pub(crate) unkeyed_bypass_count: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct HardLineIndexEntry {
    key: TextDocumentKey,
    source: Arc<str>,
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
        report.budget = HardLineIndexCacheBudgetSnapshot {
            max_entries: self.capacity,
            max_bytes: self.max_bytes,
        };
        report.entry_count = self.entries.len();
        report.estimated_bytes = self.estimated_bytes;
        report
    }

    pub(crate) fn count_and_window(
        &mut self,
        key: TextDocumentKey,
        source: Arc<str>,
        range: Range<usize>,
    ) -> (usize, Vec<HardLine>) {
        let access = self.next_access();
        if let Some(index) = self.entries.iter().position(|entry| entry.key == key) {
            let source_matches = if Arc::ptr_eq(&self.entries[index].source, &source) {
                self.report.source_pointer_hit_count =
                    self.report.source_pointer_hit_count.saturating_add(1);
                true
            } else {
                self.report.source_exact_compare_count =
                    self.report.source_exact_compare_count.saturating_add(1);
                self.entries[index].source == source
            };
            if source_matches {
                self.report.hit_count = self.report.hit_count.saturating_add(1);
                let entry = &mut self.entries[index];
                entry.last_access = access;
                return (entry.lines.len(), line_window(&entry.lines, range));
            }

            let stale = self.entries.swap_remove(index);
            self.estimated_bytes = self.estimated_bytes.saturating_sub(stale.estimated_bytes);
            self.report.stale_source_alias_count =
                self.report.stale_source_alias_count.saturating_add(1);
        }

        self.report.miss_count = self.report.miss_count.saturating_add(1);
        let mut lines = Vec::new();
        let mut window = Vec::new();
        let mut line_count = 0usize;
        let max_cached_lines = self.max_cached_line_count(source.len());
        let mut cacheable = true;
        visit_hard_lines(source.as_ref(), |line| {
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

        if !cacheable || estimated_entry_bytes(&source, &lines) > self.max_bytes {
            self.report.oversized_bypass_count =
                self.report.oversized_bypass_count.saturating_add(1);
            return (line_count, window);
        }

        let estimated_bytes = estimated_entry_bytes(&source, &lines);
        self.evict_for(estimated_bytes);
        self.estimated_bytes = self.estimated_bytes.saturating_add(estimated_bytes);
        self.entries.push(HardLineIndexEntry {
            key,
            source,
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

    fn max_cached_line_count(&self, source_len: usize) -> usize {
        self.max_bytes
            .saturating_sub(size_of::<HardLineIndexEntry>())
            .saturating_sub(source_len)
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

fn estimated_entry_bytes(source: &Arc<str>, lines: &Vec<HardLine>) -> usize {
    lines
        .capacity()
        .saturating_mul(size_of::<HardLine>())
        .saturating_add(size_of::<HardLineIndexEntry>())
        .saturating_add(source.len())
}

#[cfg(test)]
mod tests {
    use std::{mem::size_of, sync::Arc};

    use super::{HardLineIndexCache, HardLineIndexEntry, TextDocumentKey};

    #[test]
    fn hard_line_report_exposes_the_effective_cache_budget() {
        let cache = HardLineIndexCache::with_limits(3, 4096);

        assert_eq!(
            cache.report().budget,
            super::HardLineIndexCacheBudgetSnapshot {
                max_entries: 3,
                max_bytes: 4096,
            }
        );
    }
    use crate::text::HardLine;

    #[test]
    fn hard_line_index_cache_reuses_one_document_for_multiple_viewports() {
        let text: Arc<str> = Arc::from("zero\none\ntwo\nthree");
        let mut cache = HardLineIndexCache::with_limits(2, 4 * 1024);

        let key = TextDocumentKey::new(7, 1);
        let (first_count, first_window) = cache.count_and_window(key, Arc::clone(&text), 1..2);
        let (second_count, second_window) = cache.count_and_window(key, Arc::clone(&text), 3..4);

        assert_eq!(first_count, 4);
        assert_eq!(second_count, 4);
        assert_eq!(first_window[0].content, 5..8);
        assert_eq!(second_window[0].content, 13..18);
        assert_eq!(cache.report().build_count, 1);
        assert_eq!(cache.report().hit_count, 1);
    }

    #[test]
    fn hard_line_index_cache_reuses_unicode_crlf_document_windows() {
        let text: Arc<str> = Arc::from("first\r\n世界\u{2028}third");
        let mut cache = HardLineIndexCache::with_limits(2, 4 * 1024);
        let key = TextDocumentKey::new(7, 1);

        let (first_count, first_window) = cache.count_and_window(key, Arc::clone(&text), 0..1);
        let (second_count, second_window) = cache.count_and_window(key, Arc::clone(&text), 1..2);

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

        let (before_count, _) = cache.count_and_window(
            TextDocumentKey::new(7, 1),
            Arc::<str>::from("zero\none"),
            0..1,
        );
        let (after_count, after_window) = cache.count_and_window(
            TextDocumentKey::new(7, 2),
            Arc::<str>::from("zero\none\ntwo"),
            2..3,
        );

        assert_eq!(before_count, 2);
        assert_eq!(after_count, 3);
        assert_eq!(after_window[0].content, 9..12);
        assert_eq!(cache.report().build_count, 2);
    }

    #[test]
    fn hard_line_index_cache_rejects_a_same_revision_source_alias() {
        let mut cache = HardLineIndexCache::with_limits(2, 4 * 1024);
        let key = TextDocumentKey::new(7, 1);

        let (before_count, _) = cache.count_and_window(key, Arc::<str>::from("aa\nbbbb"), 0..1);
        let (after_count, after_window) =
            cache.count_and_window(key, Arc::<str>::from("aaaa\nbb"), 1..2);

        assert_eq!(before_count, 2);
        assert_eq!(after_count, 2);
        assert_eq!(after_window[0].content, 5..7);
        let report = cache.report();
        assert_eq!(report.build_count, 2);
        assert_eq!(report.hit_count, 0);
        assert_eq!(report.stale_source_alias_count, 1);
    }

    #[test]
    fn hard_line_index_cache_uses_pointer_identity_for_a_shared_source() {
        let mut cache = HardLineIndexCache::with_limits(2, 4 * 1024);
        let key = TextDocumentKey::new(7, 1);
        let source: Arc<str> = Arc::from("zero\none\ntwo");

        cache.count_and_window(key, Arc::clone(&source), 0..1);
        cache.count_and_window(key, Arc::clone(&source), 1..2);

        let report = cache.report();
        assert_eq!(report.build_count, 1);
        assert_eq!(report.hit_count, 1);
        assert_eq!(report.source_pointer_hit_count, 1);
        assert_eq!(report.source_exact_compare_count, 0);
    }

    #[test]
    fn hard_line_index_cache_evicts_the_least_recent_document_at_capacity() {
        let mut cache = HardLineIndexCache::with_limits(2, 4 * 1024);
        let first = TextDocumentKey::new(1, 1);
        let second = TextDocumentKey::new(2, 1);
        let third = TextDocumentKey::new(3, 1);

        cache.count_and_window(first, Arc::<str>::from("first"), 0..1);
        cache.count_and_window(second, Arc::<str>::from("second"), 0..1);
        cache.count_and_window(first, Arc::<str>::from("first"), 0..1);
        cache.count_and_window(third, Arc::<str>::from("third"), 0..1);
        let (count, window) = cache.count_and_window(second, Arc::<str>::from("second"), 0..1);

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
        let source_bytes = "second".len() + "third".len();
        let mut cache = HardLineIndexCache::with_limits(8, (entry_bytes * 2) + source_bytes);
        let first = TextDocumentKey::new(1, 1);
        let second = TextDocumentKey::new(2, 1);
        let third = TextDocumentKey::new(3, 1);

        cache.count_and_window(first, Arc::<str>::from("first"), 0..1);
        cache.count_and_window(second, Arc::<str>::from("second"), 0..1);
        cache.count_and_window(first, Arc::<str>::from("first"), 0..1);
        cache.count_and_window(third, Arc::<str>::from("third"), 0..1);
        let (count, window) = cache.count_and_window(second, Arc::<str>::from("second"), 0..1);

        assert_eq!(count, 1);
        assert_eq!(window[0].content, 0..6);
        assert_eq!(cache.report().entry_count, 2);
        assert_eq!(
            cache.report().estimated_bytes,
            (entry_bytes * 2) + source_bytes
        );
        assert_eq!(cache.report().build_count, 4);
        assert_eq!(cache.report().hit_count, 1);
        assert_eq!(cache.report().evicted_count, 2);
    }

    #[test]
    fn oversized_documents_return_only_the_requested_window_without_retaining_an_index() {
        let mut cache = HardLineIndexCache::with_limits(2, size_of::<HardLine>());

        let (line_count, window) = cache.count_and_window(
            TextDocumentKey::new(7, 1),
            Arc::<str>::from("zero\none\ntwo"),
            1..2,
        );

        assert_eq!(line_count, 3);
        assert_eq!(window[0].content, 5..8);
        assert_eq!(cache.report().entry_count, 0);
        assert_eq!(cache.report().build_count, 0);
        assert_eq!(cache.report().oversized_bypass_count, 1);
    }
}
