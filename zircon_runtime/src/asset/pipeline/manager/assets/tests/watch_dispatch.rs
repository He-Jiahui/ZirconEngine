use std::hint::black_box;
use std::sync::Mutex;
use std::time::Instant;

use super::*;

fn change(index: usize) -> AssetChange {
    AssetChange::new(
        AssetChangeKind::Modified,
        AssetUri::parse(&format!("res://data/watch-{index}.json")).unwrap(),
        None,
    )
}

fn state() -> ProjectWatcherActivationState {
    ProjectWatcherActivationState {
        lifecycle: ProjectWatcherLifecycle::Pending,
        changes: Vec::new(),
        coalescible_change_indices: Default::default(),
        queued_change_bytes: 0,
        requires_reconciliation: false,
        diagnostics: AssetWatchBatchDiagnostics::default(),
        errors: Default::default(),
        worker_scheduled: false,
    }
}

#[test]
fn activation_coalesces_modifications_without_crossing_remove_edges() {
    let uri = AssetUri::parse("res://data/watch.json").unwrap();
    let mut state = state();
    merge_batch(
        &mut state,
        AssetWatchBatch {
            changes: vec![
                AssetChange::new(AssetChangeKind::Modified, uri.clone(), None),
                AssetChange::new(AssetChangeKind::Modified, uri.clone(), None),
                AssetChange::new(AssetChangeKind::Removed, uri.clone(), None),
                AssetChange::new(AssetChangeKind::Added, uri, None),
            ],
            ..AssetWatchBatch::default()
        },
    );

    assert_eq!(state.changes.len(), 3);
    assert_eq!(state.changes[0].kind, AssetChangeKind::Modified);
    assert_eq!(state.changes[1].kind, AssetChangeKind::Removed);
    assert_eq!(state.changes[2].kind, AssetChangeKind::Added);
}

#[test]
fn activation_worker_admission_is_singleflight() {
    let mut state = state();
    state.lifecycle = ProjectWatcherLifecycle::Active;
    state.changes.push(change(0));

    assert!(should_schedule_worker(&mut state));
    assert!(!should_schedule_worker(&mut state));
}

#[test]
fn activation_entry_overflow_discards_partial_queue_and_marks_dirty() {
    let mut state = state();
    merge_batch(
        &mut state,
        AssetWatchBatch {
            changes: (0..=WATCH_ACTIVATION_ENTRY_CAPACITY).map(change).collect(),
            ..AssetWatchBatch::default()
        },
    );

    assert!(state.requires_reconciliation);
    assert!(state.changes.is_empty());
    assert_eq!(state.queued_change_bytes, 0);
    assert_eq!(state.diagnostics.pending_overflow_count, 1);
}

#[test]
fn activation_error_overflow_discards_oldest_and_preserves_fifo_order() {
    let manager = ProjectAssetManager::default();
    let activation = Arc::new(ProjectWatcherActivation {
        state: Mutex::new(state()),
    });
    for index in 0..(WATCH_ACTIVATION_ERROR_CAPACITY + 2) {
        activation.enqueue_error(
            &manager,
            AssetWatchError::from_message(
                std::path::PathBuf::from("project-assets"),
                format!("watch-error-{index}"),
            ),
        );
    }

    let (batch, errors) = activation
        .take_work()
        .expect("watch errors should be queued");
    assert!(batch.requires_reconciliation);
    assert_eq!(errors.len(), WATCH_ACTIVATION_ERROR_CAPACITY);
    assert_eq!(errors.front().unwrap().message, "watch-error-2");
    assert_eq!(
        errors.back().unwrap().message,
        format!("watch-error-{}", WATCH_ACTIVATION_ERROR_CAPACITY + 1)
    );
}

#[test]
#[ignore = "managed release performance evidence"]
fn watch_error_tail_queue_release_benchmark_evidence() {
    const ITEMS: usize = 200_000;
    const SAMPLE_PAIRS: usize = 21;

    let mut legacy_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            let mut queue = Vec::with_capacity(WATCH_ACTIVATION_ERROR_CAPACITY);
            for item in 0..ITEMS {
                if queue.len() == WATCH_ACTIVATION_ERROR_CAPACITY {
                    black_box(queue.remove(0));
                }
                queue.push(item);
            }
            black_box(queue);
            legacy_samples_ns.push(started.elapsed().as_nanos());
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            let mut queue =
                std::collections::VecDeque::with_capacity(WATCH_ACTIVATION_ERROR_CAPACITY);
            for item in 0..ITEMS {
                black_box(push_bounded_error(
                    &mut queue,
                    item,
                    WATCH_ACTIVATION_ERROR_CAPACITY,
                ));
            }
            black_box(queue);
            optimized_samples_ns.push(started.elapsed().as_nanos());
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples_ns, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples_ns, 95);
    let overflow_count = ITEMS - WATCH_ACTIVATION_ERROR_CAPACITY;
    let legacy_moves = overflow_count * (WATCH_ACTIVATION_ERROR_CAPACITY - 1);
    println!(
        "WATCH_ERROR_TAIL_QUEUE_BENCH_V1 items={ITEMS} capacity={} sample_pairs={SAMPLE_PAIRS} \
             overflow_count={overflow_count} legacy_moves={legacy_moves} optimized_moves=0 \
             legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} \
             optimized_ns={}",
        WATCH_ACTIVATION_ERROR_CAPACITY,
        join_nanosecond_samples(&legacy_samples_ns),
        join_nanosecond_samples(&optimized_samples_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns.saturating_mul(3),
        "optimized P95 {optimized_p95_ns}ns must be at most 75% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn join_nanosecond_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    assert!(!samples.is_empty());
    assert!((1..=100).contains(&percentile));
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let index = (ordered.len() * percentile).div_ceil(100) - 1;
    ordered[index]
}
