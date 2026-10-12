use std::collections::VecDeque;
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use super::{
    move_change_key_to_tail, EditorAssetChangeHub, EditorAssetChangeKey, EditorAssetChangeKind,
    EditorAssetChangeRecord, MAX_PENDING_EDITOR_ASSET_CHANGES,
};

fn legacy_move_change_key_to_tail(
    order: &mut VecDeque<EditorAssetChangeKey>,
    key: EditorAssetChangeKey,
) {
    order.retain(|pending_key| pending_key != &key);
    order.push_back(key);
}

fn benchmark_key(index: usize) -> EditorAssetChangeKey {
    EditorAssetChangeKey::Asset {
        kind: EditorAssetChangeKind::PreviewChanged,
        uuid: Some(format!("asset-{index:04}")),
        locator: Some(format!("res://asset-{index:04}.asset")),
    }
}

#[test]
fn same_asset_preview_storm_coalesces_to_latest_revision() {
    let hub = EditorAssetChangeHub::default();
    let subscription = hub.subscribe();
    for revision in 0..10_000 {
        hub.publish(change(
            EditorAssetChangeKind::PreviewChanged,
            revision,
            Some("asset-a"),
        ));
    }

    assert_eq!(subscription.pending_len(), 1);
    let delivery = subscription.try_recv().expect("latest preview change");
    assert_eq!(delivery.change.catalog_revision, 9_999);
    assert!(subscription.try_recv().is_none());
}

#[test]
fn overflow_collapses_to_latest_catalog_generation() {
    let hub = EditorAssetChangeHub::default();
    let subscription = hub.subscribe();
    for revision in 0..=MAX_PENDING_EDITOR_ASSET_CHANGES as u64 {
        hub.publish(change(
            EditorAssetChangeKind::ReferenceChanged,
            revision,
            Some(&format!("asset-{revision}")),
        ));
    }

    assert_eq!(subscription.pending_len(), 1);
    let delivery = subscription.try_recv().expect("overflow fallback");
    assert_eq!(delivery.change.kind, EditorAssetChangeKind::CatalogChanged);
    assert_eq!(
        delivery.change.catalog_revision,
        MAX_PENDING_EDITOR_ASSET_CHANGES as u64
    );
}

#[test]
fn fanout_shares_one_immutable_change_payload() {
    let hub = EditorAssetChangeHub::default();
    let left = hub.subscribe();
    let right = hub.subscribe();
    hub.publish(change(
        EditorAssetChangeKind::PreviewChanged,
        7,
        Some("asset-a"),
    ));

    let left = left.try_recv().expect("left delivery");
    let right = right.try_recv().expect("right delivery");
    assert!(Arc::ptr_eq(&left.change, &right.change));
}

#[test]
fn wake_subscription_notifies_after_a_change_enters_its_mailbox() {
    let hub = EditorAssetChangeHub::default();
    let wake_count = Arc::new(AtomicUsize::new(0));
    let wake_count_for_callback = Arc::clone(&wake_count);
    let subscription = hub.subscribe_with_wake(Arc::new(move || {
        wake_count_for_callback.fetch_add(1, Ordering::Relaxed);
    }));

    hub.publish(change(
        EditorAssetChangeKind::PreviewChanged,
        7,
        Some("asset-a"),
    ));

    assert_eq!(wake_count.load(Ordering::Relaxed), 1);
    assert_eq!(subscription.pending_len(), 1);
}

#[test]
fn coalesced_key_moves_to_tail_without_revision_regression() {
    let hub = EditorAssetChangeHub::default();
    let subscription = hub.subscribe();
    hub.publish(change(
        EditorAssetChangeKind::PreviewChanged,
        1,
        Some("asset-a"),
    ));
    hub.publish(change(
        EditorAssetChangeKind::PreviewChanged,
        2,
        Some("asset-b"),
    ));
    hub.publish(change(
        EditorAssetChangeKind::PreviewChanged,
        3,
        Some("asset-a"),
    ));

    let first = subscription.try_recv().expect("asset-b");
    let second = subscription.try_recv().expect("newer asset-a");
    assert_eq!(first.change.uuid.as_deref(), Some("asset-b"));
    assert_eq!(first.change.catalog_revision, 2);
    assert_eq!(second.change.uuid.as_deref(), Some("asset-a"));
    assert_eq!(second.change.catalog_revision, 3);
}

#[test]
fn optimization_batch_eq_tail_coalescing_preserves_existing_queue_order() {
    let seed = (0..MAX_PENDING_EDITOR_ASSET_CHANGES)
        .map(benchmark_key)
        .collect::<VecDeque<_>>();
    for target_index in [
        0,
        MAX_PENDING_EDITOR_ASSET_CHANGES / 2,
        MAX_PENDING_EDITOR_ASSET_CHANGES - 1,
    ] {
        let key = benchmark_key(target_index);
        let mut legacy = seed.clone();
        let mut optimized = seed.clone();

        legacy_move_change_key_to_tail(&mut legacy, key.clone());
        move_change_key_to_tail(&mut optimized, key);

        assert_eq!(optimized, legacy);
    }

    let source = include_str!("../change_stream.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("asset change stream production implementation");
    let movement = production
        .split("fn move_change_key_to_tail(")
        .nth(1)
        .expect("coalesced key movement");
    assert!(movement.contains("order.back() == Some(&key)"));
}

#[test]
#[ignore = "release-only tail coalescing fast-path benchmark"]
fn optimization_batch_eq_tail_coalescing_release_benchmark_evidence() {
    const SAMPLE_PAIRS: usize = 17;
    const MOVES_PER_SAMPLE: usize = 8_192;

    fn measure_legacy(seed: &VecDeque<EditorAssetChangeKey>, key: &EditorAssetChangeKey) -> u128 {
        let mut order = seed.clone();
        let started = Instant::now();
        for _ in 0..MOVES_PER_SAMPLE {
            legacy_move_change_key_to_tail(&mut order, black_box(key.clone()));
        }
        black_box(order.len());
        started.elapsed().as_nanos().max(1)
    }

    fn measure_optimized(
        seed: &VecDeque<EditorAssetChangeKey>,
        key: &EditorAssetChangeKey,
    ) -> u128 {
        let mut order = seed.clone();
        let started = Instant::now();
        for _ in 0..MOVES_PER_SAMPLE {
            move_change_key_to_tail(&mut order, black_box(key.clone()));
        }
        black_box(order.len());
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    let seed = (0..MAX_PENDING_EDITOR_ASSET_CHANGES)
        .map(benchmark_key)
        .collect::<VecDeque<_>>();
    let key = seed.back().expect("full benchmark mailbox").clone();
    for _ in 0..4 {
        black_box(measure_legacy(&seed, &key));
        black_box(measure_optimized(&seed, &key));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
            legacy_samples.push(measure_legacy(&seed, &key));
            optimized_samples.push(measure_optimized(&seed, &key));
        } else {
            optimized_samples.push(measure_optimized(&seed, &key));
            legacy_samples.push(measure_legacy(&seed, &key));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR379_TAIL_COALESCING_FAST_PATH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             moves_per_sample={MOVES_PER_SAMPLE} mailbox_size={MAX_PENDING_EDITOR_ASSET_CHANGES} \
             pair_order=alternating_legacy_even legacy_queue_scans_per_move=1 \
             optimized_queue_scans_per_tail_move=0 legacy_p50_ns={legacy_p50_ns} \
             optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(20),
        "tail coalescing must reduce P95 by at least 80%: \
             legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

#[test]
fn concurrent_publishers_converge_all_subscribers_to_same_latest_payload() {
    let hub = EditorAssetChangeHub::default();
    let left = hub.subscribe();
    let right = hub.subscribe();
    let publishers = (0..4)
        .map(|_| {
            let hub = hub.clone();
            std::thread::spawn(move || {
                for _ in 0..1_000 {
                    hub.publish(change(
                        EditorAssetChangeKind::PreviewChanged,
                        7,
                        Some("asset-a"),
                    ));
                }
            })
        })
        .collect::<Vec<_>>();
    for publisher in publishers {
        publisher.join().expect("publisher");
    }

    let left_delivery = left.try_recv().expect("left latest");
    let right_delivery = right.try_recv().expect("right latest");
    assert!(Arc::ptr_eq(&left_delivery.change, &right_delivery.change));
    assert!(left.try_recv().is_none());
    assert!(right.try_recv().is_none());
}

#[test]
fn discarded_or_completed_delivery_is_not_implicitly_requeued() {
    let hub = EditorAssetChangeHub::default();
    let subscription = hub.subscribe();
    hub.publish(change(
        EditorAssetChangeKind::PreviewAdmissionAvailable,
        3,
        Some("asset-a"),
    ));

    assert_eq!(subscription.discard_pending(), 1);
    assert_eq!(subscription.pending_len(), 0);
    assert!(subscription.try_recv().is_none());
}

#[test]
fn silent_subscribe_drop_churn_prunes_dead_owners() {
    let hub = EditorAssetChangeHub::default();
    for _ in 0..10_000 {
        drop(hub.subscribe());
    }

    let live = hub.subscribe();
    assert_eq!(hub.lock_subscribers().len(), 1);
    drop(live);
}

fn change(
    kind: EditorAssetChangeKind,
    catalog_revision: u64,
    uuid: Option<&str>,
) -> EditorAssetChangeRecord {
    EditorAssetChangeRecord {
        kind,
        catalog_revision,
        uuid: uuid.map(str::to_string),
        locator: uuid.map(|uuid| format!("res://{uuid}.asset")),
    }
}
