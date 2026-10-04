use std::collections::VecDeque;
use std::time::{Duration, Instant};

use super::{
    transaction_record_by_id, EditWorldRoute, SelectionSnapshot, TransactionId, TransactionRecord,
    WorldDomain,
};

fn transaction_record(sequence: u64) -> TransactionRecord {
    TransactionRecord {
        id: TransactionId::from_sequence(sequence),
        label: String::new(),
        timestamp_frame: sequence,
        route: EditWorldRoute::logical(WorldDomain::Edit),
        commands: Vec::new(),
        participants: Default::default(),
        selection_before: SelectionSnapshot::default(),
        selection_after: SelectionSnapshot::default(),
        significant: true,
    }
}

#[test]
fn undo_and_redo_return_compact_event_metadata_without_copying_detail_records() {
    let source = include_str!("../history.rs");
    let undo_body = source
        .split("pub(crate) fn undo")
        .nth(1)
        .and_then(|body| body.split("pub(crate) fn redo").next())
        .expect("undo body should remain available");
    let redo_body = source
        .split("pub(crate) fn redo")
        .nth(1)
        .and_then(|body| body.split("pub(crate) fn mark_saved_current").next())
        .expect("redo body should remain available");
    let full_detail = ["record", ".detail()"].concat();

    assert!(!undo_body.contains(&full_detail));
    assert!(!redo_body.contains(&full_detail));
}

#[test]
fn optimization_wave_20260824e_editor03_history_lookup_handles_wrapped_storage_and_id_gaps() {
    let mut entries = VecDeque::with_capacity(8);
    for sequence in [10, 20, 30, 40, 50, 60, 70, 80] {
        entries.push_back(transaction_record(sequence));
    }
    for _ in 0..4 {
        entries.pop_front();
    }
    for sequence in [90, 100, 110, 120] {
        entries.push_back(transaction_record(sequence));
    }
    let (front, back) = entries.as_slices();

    assert!(!front.is_empty());
    assert!(!back.is_empty());
    assert_eq!(
        transaction_record_by_id(&entries, TransactionId::from_sequence(50))
            .map(|record| record.id.raw()),
        Some(50)
    );
    assert_eq!(
        transaction_record_by_id(&entries, TransactionId::from_sequence(120))
            .map(|record| record.id.raw()),
        Some(120)
    );
    assert!(transaction_record_by_id(&entries, TransactionId::from_sequence(55)).is_none());
}

#[test]
fn optimization_wave_20260824e_editor03_history_journal_lookup_is_logarithmic() {
    let source = include_str!("../history.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("history implementation");
    let journal = production
        .split("impl HistoryStore {")
        .nth(1)
        .and_then(|source| source.split("pub(crate) fn journal").nth(1))
        .and_then(|source| source.split("pub(crate) fn mark_saved_current").next())
        .expect("history journal implementation");

    assert!(journal.contains("transaction_record_by_id"));
    assert!(production.contains("binary_search_by_key"));
    assert!(!journal.contains("self.entries.iter().find"));
}

#[test]
#[ignore = "managed release evidence"]
fn optimization_wave_20260824e_editor03_history_journal_lookup_evidence() {
    const ENTRY_COUNT: usize = 100_000;
    const LOOKUP_COUNT: usize = 100_000;
    const TARGET: Duration = Duration::from_secs(1);

    let mut entries = VecDeque::with_capacity(ENTRY_COUNT);
    for sequence in 0..ENTRY_COUNT as u64 {
        entries.push_back(transaction_record(sequence));
    }
    for _ in 0..ENTRY_COUNT / 2 {
        entries.pop_front();
    }
    for sequence in ENTRY_COUNT as u64..ENTRY_COUNT as u64 + ENTRY_COUNT as u64 / 2 {
        entries.push_back(transaction_record(sequence));
    }
    let target = TransactionId::from_sequence(ENTRY_COUNT as u64 + ENTRY_COUNT as u64 / 2 - 1);
    let (front, back) = entries.as_slices();
    let binary_comparisons_per_lookup =
        binary_search_comparison_bound(front.len()) + binary_search_comparison_bound(back.len());

    let started = Instant::now();
    let mut checksum = 0_u64;
    for _ in 0..LOOKUP_COUNT {
        let record = transaction_record_by_id(&entries, std::hint::black_box(target))
            .expect("tail transaction");
        checksum = checksum.wrapping_add(std::hint::black_box(record.id.raw()));
    }
    let elapsed = started.elapsed();
    let comparisons_before = ENTRY_COUNT * LOOKUP_COUNT;
    let comparisons_after_bound = binary_comparisons_per_lookup * LOOKUP_COUNT;
    let comparison_reduction_percent =
        (1.0 - comparisons_after_bound as f64 / comparisons_before as f64) * 100.0;

    assert_ne!(checksum, 0);
    assert!(elapsed <= TARGET, "elapsed={elapsed:?} target={TARGET:?}");
    println!(
        "EDITOR03_HISTORY_LOOKUP_BENCH_V1 entries={} lookups={} comparisons_before={} comparisons_after_bound={} comparison_reduction_percent={:.4} elapsed_ns={} target_ns={}",
        ENTRY_COUNT,
        LOOKUP_COUNT,
        comparisons_before,
        comparisons_after_bound,
        comparison_reduction_percent,
        elapsed.as_nanos(),
        TARGET.as_nanos()
    );
}

fn binary_search_comparison_bound(len: usize) -> usize {
    if len == 0 {
        0
    } else {
        (usize::BITS - len.leading_zeros()) as usize
    }
}
