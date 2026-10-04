use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

use super::*;

#[test]
fn optimization_batch_20260826n_editor23_single_pass_enqueue_preserves_queue_semantics() {
    let now = Instant::now();
    let mut queue = UiAssetRefreshQueue::default();
    assert!(queue.defer_retry_at(BTreeSet::from(["a.zui".to_string()]), 0, 1, now,));

    assert!(queue.enqueue(vec![
        "b.zui".to_string(),
        "a.zui".to_string(),
        "a.zui".to_string(),
    ]));
    assert_eq!(queue.snapshot().latest_generation, 1);
    assert_eq!(queue.snapshot().deferred_retry_count, 0);
    assert_eq!(queue.snapshot().pending_asset_count, 2);

    let request = queue.start_next_at(now).expect("queued refresh request");
    assert_eq!(request.retry_attempt, 0);
    assert_eq!(request.generation, 1);
    assert_eq!(
        request.changed_asset_ids,
        BTreeSet::from(["a.zui".to_string(), "b.zui".to_string()])
    );
}

#[test]
fn optimization_batch_20260826n_editor23_enqueue_admits_directly_to_pending_tree() {
    let source = include_str!("../queue.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("asset refresh queue production source");
    let enqueue = production
        .split("pub(super) fn enqueue")
        .nth(1)
        .expect("refresh enqueue implementation")
        .split("pub(super) fn start_next")
        .next()
        .expect("bounded refresh enqueue implementation");

    assert!(enqueue.contains("let mut received_any = false"));
    assert!(enqueue.contains("for asset_id in changed_asset_ids"));
    assert!(enqueue.contains("self.deferred_retries.remove(&asset_id)"));
    assert!(enqueue.contains("self.pending_asset_ids.insert(asset_id)"));
    assert!(!enqueue.contains("collect::<BTreeSet<_>>()"));
    assert!(!enqueue.contains("self.pending_asset_ids.extend(changed_asset_ids)"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_20260826n_editor23_refresh_enqueue_single_pass_performance_evidence() {
    fn legacy_enqueue(entries: Vec<String>) -> BTreeSet<String> {
        let changed_asset_ids = entries.into_iter().collect::<BTreeSet<_>>();
        let mut pending_asset_ids = BTreeSet::new();
        pending_asset_ids.extend(changed_asset_ids);
        pending_asset_ids
    }

    let entries = (0..32_768)
        .map(|index| format!("res://editor/ui/refresh/asset_{index:05}.zui"))
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(17);
    let mut single_pass_samples = Vec::with_capacity(17);
    for _ in 0..17 {
        let legacy_input = entries.clone();
        let single_pass_input = entries.clone();

        let started = Instant::now();
        black_box(legacy_enqueue(black_box(legacy_input)));
        legacy_samples.push(started.elapsed().as_nanos());

        let mut queue = UiAssetRefreshQueue::default();
        let started = Instant::now();
        black_box(queue.enqueue(black_box(single_pass_input)));
        black_box(queue.pending_asset_ids.len());
        single_pass_samples.push(started.elapsed().as_nanos());
    }

    legacy_samples.sort_unstable();
    single_pass_samples.sort_unstable();
    let legacy_p95 = legacy_samples[16];
    let single_pass_p95 = single_pass_samples[16];
    println!(
        "EDITOR23_REFRESH_QUEUE_SINGLE_PASS_ADMISSION_BENCH_V1 asset_ids={} legacy_p95_ns={} single_pass_p95_ns={} legacy_tree_admissions={} single_pass_tree_admissions={} legacy_intermediate_tree_nodes={} single_pass_intermediate_tree_nodes=0 target_ratio_bp=6000",
        entries.len(),
        legacy_p95,
        single_pass_p95,
        entries.len() * 2,
        entries.len(),
        entries.len(),
    );
    assert!(
        single_pass_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(6_000),
        "single-pass refresh enqueue P95 {single_pass_p95} ns exceeded 60% of legacy {legacy_p95} ns"
    );
}
