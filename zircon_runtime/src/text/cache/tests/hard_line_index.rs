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
