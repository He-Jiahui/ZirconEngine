use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

use crate::core::editing::engine::{
    HistoryContextId, HistoryDetailPage, HistoryRecordDetail, HistoryStatus, SelectionSnapshot,
    TransactionId,
};
use crate::core::editor_message::DocumentId;
use crate::ui::workbench::snapshot::TransactionHistorySnapshot;

fn record(sequence: u64, label: &str) -> HistoryRecordDetail {
    HistoryRecordDetail {
        id: TransactionId::from_sequence(sequence),
        label: label.to_string(),
        timestamp_frame: sequence * 10,
        command_count: sequence as usize,
        participants: BTreeSet::new(),
        selection_before: SelectionSnapshot::default(),
        selection_after: SelectionSnapshot::default(),
        significant: true,
    }
}

#[test]
fn history_projection_marks_the_applied_and_redo_segments_from_the_authoritative_top() {
    let history = HistoryContextId::Document(DocumentId::new(41));
    let page = HistoryDetailPage::new(
        HistoryStatus {
            len: 3,
            top: Some(TransactionId::from_sequence(2)),
            saved_top: Some(TransactionId::from_sequence(1)),
            saved_top_reachable: true,
            can_undo: true,
            can_redo: true,
            dirty: true,
            generation: 7,
        },
        vec![record(1, "Create"), record(2, "Rename"), record(3, "Move")],
        None,
    );

    let snapshot = TransactionHistorySnapshot::from_page(history, page);

    assert_eq!(snapshot.context, history);
    assert_eq!(snapshot.generation, 7);
    assert_eq!(snapshot.total_count, 3);
    assert!(!snapshot.truncated);
    assert_eq!(
        snapshot
            .rows
            .iter()
            .map(|row| (
                row.label.as_str(),
                row.applied,
                row.is_top,
                row.is_saved_top
            ))
            .collect::<Vec<_>>(),
        vec![
            ("Create", true, false, true),
            ("Rename", true, true, false),
            ("Move", false, false, false),
        ]
    );
}

#[test]
fn history_projection_reports_a_bounded_page_without_inventing_application_state() {
    let history = HistoryContextId::Global;
    let page = HistoryDetailPage::new(
        HistoryStatus {
            len: 130,
            top: None,
            saved_top: None,
            saved_top_reachable: true,
            can_undo: false,
            can_redo: true,
            dirty: false,
            generation: 9,
        },
        vec![record(90, "Oldest visible"), record(91, "Next visible")],
        None,
    );

    let snapshot = TransactionHistorySnapshot::from_page(history, page);

    assert_eq!(snapshot.rows.len(), 2);
    assert!(snapshot.truncated);
    assert!(snapshot.rows.iter().all(|row| !row.applied));
    assert!(snapshot.can_redo);
    assert!(!snapshot.can_undo);
}

#[test]
fn history_projection_marks_the_visible_prefix_applied_when_top_is_on_a_later_page() {
    let history = HistoryContextId::Global;
    let page = HistoryDetailPage::new(
        HistoryStatus {
            len: 130,
            top: Some(TransactionId::from_sequence(130)),
            saved_top: None,
            saved_top_reachable: true,
            can_undo: true,
            can_redo: false,
            dirty: true,
            generation: 10,
        },
        vec![record(1, "Oldest visible"), record(2, "Next visible")],
        None,
    );

    let snapshot = TransactionHistorySnapshot::from_page(history, page);

    assert!(snapshot.truncated);
    assert!(snapshot.rows.iter().all(|row| row.applied));
    assert!(snapshot.rows.iter().all(|row| !row.is_top));
}

#[test]
fn history_projection_preserves_tail_top_semantics_for_a_large_ordered_page() {
    const RECORD_COUNT: usize = 16_384;
    let history = HistoryContextId::Global;
    let records = (0..RECORD_COUNT)
        .map(|sequence| record(sequence as u64 + 1, "ordered"))
        .collect();
    let page = HistoryDetailPage::new(
        HistoryStatus {
            len: RECORD_COUNT,
            top: Some(TransactionId::from_sequence(RECORD_COUNT as u64)),
            saved_top: None,
            saved_top_reachable: true,
            can_undo: true,
            can_redo: false,
            dirty: true,
            generation: 11,
        },
        records,
        None,
    );

    let snapshot = TransactionHistorySnapshot::from_page(history, page);

    assert_eq!(snapshot.rows.len(), RECORD_COUNT);
    assert!(snapshot.rows[..RECORD_COUNT - 1]
        .iter()
        .all(|row| row.applied && !row.is_top));
    assert!(snapshot
        .rows
        .last()
        .is_some_and(|row| row.applied && row.is_top));
}

#[test]
#[ignore = "managed release evidence"]
fn history_projection_binary_top_lookup_release_benchmark() {
    const RECORD_COUNT: usize = 16_384;
    const SAMPLE_PAIRS: usize = 17;
    let ids = (0..RECORD_COUNT)
        .map(|sequence| TransactionId::from_sequence(sequence as u64 + 1))
        .collect::<Vec<_>>();
    let target = *ids.last().expect("benchmark has a tail record");

    let mut legacy = || {
        ids.iter()
            .position(|candidate| *candidate == black_box(target))
            .expect("legacy lookup finds the tail")
    };
    let mut optimized = || {
        ids.binary_search_by_key(&black_box(target), |candidate| *candidate)
            .expect("binary lookup finds the tail")
    };
    assert_eq!(legacy(), RECORD_COUNT - 1);
    assert_eq!(optimized(), RECORD_COUNT - 1);

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_lookup_ns(&mut legacy));
            optimized_ns.push(measure_lookup_ns(&mut optimized));
        } else {
            optimized_ns.push(measure_lookup_ns(&mut optimized));
            legacy_ns.push(measure_lookup_ns(&mut legacy));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns,
        "binary top lookup P95 must be at least 75% below the linear tail scan: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
    println!(
        "EDITOR02_HISTORY_SNAPSHOT_BINARY_TOP_BENCH_V1 records={RECORD_COUNT} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_steps_per_lookup={RECORD_COUNT} optimized_steps_per_lookup={} legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        usize::BITS - RECORD_COUNT.leading_zeros(),
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn measure_lookup_ns(operation: &mut impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
