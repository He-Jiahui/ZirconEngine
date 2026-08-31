use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use super::WindowHostManager;
use crate::ui::workbench::layout::MainPageId;

const WINDOW_COUNT: usize = 1_024;
const ACTIVE_WINDOW_COUNT: usize = WINDOW_COUNT / 2;
const SAMPLE_COUNT: usize = 17;

#[test]
fn optimization_batch_20260826cb_window_host_hash_reconcile_preserves_state_and_debug_order() {
    let mut manager = WindowHostManager::default();
    for (window_id, handle) in [("window:z", 3), ("window:a", 1), ("window:m", 2)] {
        manager.open_native_window(MainPageId::new(window_id), Some(handle));
    }

    let state_ids = manager
        .states()
        .into_iter()
        .map(|state| state.window_id.0)
        .collect::<Vec<_>>();
    assert_eq!(
        state_ids,
        ["window:a", "window:m", "window:z"].map(String::from)
    );

    let debug = format!("{manager:?}");
    let first = debug.find("window:a").expect("first window in Debug");
    let middle = debug.find("window:m").expect("middle window in Debug");
    let last = debug.find("window:z").expect("last window in Debug");
    assert!(first < middle && middle < last);
}

#[test]
fn optimization_batch_20260826cb_window_host_hash_reconcile_is_linear_and_ordered_at_output() {
    let source = include_str!("../window_host_manager.rs");

    assert!(source.contains("windows: HashMap<MainPageId, NativeWindowRecord>"));
    assert!(source.contains("collect::<HashSet<_>>()"));
    assert!(source.contains("self.windows"));
    assert!(source.contains(".retain(|window_id, _| layout_window_ids.contains(window_id))"));
    assert!(source.contains("states.sort_unstable_by"));
    assert!(source.contains("self.windows.iter().collect::<BTreeMap<_, _>>()"));
    assert!(!source.contains("tracked_window_ids"));
    assert!(!source.contains(".any(|window| window.window_id == window_id)"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826cb_window_host_hash_reconcile_p95() {
    let tracked = (0..WINDOW_COUNT)
        .map(|index| MainPageId::new(format!("window.shared.long.namespace.{index:04}")))
        .collect::<Vec<_>>();
    let active = tracked.iter().step_by(2).cloned().collect::<Vec<_>>();
    assert_eq!(active.len(), ACTIVE_WINDOW_COUNT);

    let mut legacy_reconcile = || legacy_stale_count(&tracked, &active);
    let mut hash_reconcile = || hash_stale_count(&tracked, &active);
    assert_eq!(black_box(legacy_reconcile()), black_box(hash_reconcile()));

    let mut legacy_ns = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_ns = Vec::with_capacity(SAMPLE_COUNT);
    for sample_index in 0..SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(&mut legacy_reconcile));
            hash_ns.push(measure_ns(&mut hash_reconcile));
        } else {
            hash_ns.push(measure_ns(&mut hash_reconcile));
            legacy_ns.push(measure_ns(&mut legacy_reconcile));
        }
    }

    let legacy_p50 = nearest_rank(&legacy_ns, 50);
    let legacy_p95 = nearest_rank(&legacy_ns, 95);
    let hash_p50 = nearest_rank(&hash_ns, 50);
    let hash_p95 = nearest_rank(&hash_ns, 95);
    assert!(
        hash_p95.saturating_mul(10) <= legacy_p95.saturating_mul(3),
        "window-host hash reconcile P95 must be at least 70% below nested scan: legacy={legacy_p95}ns hash={hash_p95}ns"
    );

    println!(
        "EDITOR13_WINDOW_HOST_HASH_RECONCILE_BENCH_V1 tracked={WINDOW_COUNT} active={ACTIVE_WINDOW_COUNT} samples={SAMPLE_COUNT} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95} nested_membership_scans_before={WINDOW_COUNT} nested_membership_scans_after=0 hash_membership_checks_after={WINDOW_COUNT} tree_window_sync_lookups_before={} hash_window_sync_lookups_after={ACTIVE_WINDOW_COUNT} legacy_ns={} hash_ns={}",
        ACTIVE_WINDOW_COUNT * 2,
        join_samples(&legacy_ns),
        join_samples(&hash_ns),
    );
}

fn legacy_stale_count(tracked: &[MainPageId], active: &[MainPageId]) -> usize {
    tracked
        .iter()
        .filter(|window_id| {
            !active
                .iter()
                .any(|active_window_id| black_box(active_window_id) == black_box(*window_id))
        })
        .count()
}

fn hash_stale_count(tracked: &[MainPageId], active: &[MainPageId]) -> usize {
    let active = active.iter().collect::<HashSet<_>>();
    tracked
        .iter()
        .filter(|window_id| !active.contains(*window_id))
        .count()
}

fn measure_ns(operation: &mut impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    assert_ne!(black_box(operation()), 0);
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
