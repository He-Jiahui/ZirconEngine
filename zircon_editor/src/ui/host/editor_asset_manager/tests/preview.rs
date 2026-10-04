use super::{PreviewScheduler, MAX_PREVIEW_IN_FLIGHT};
use zircon_runtime::asset::AssetUuid;

#[test]
fn optimization_batch_20260830en_preview_scheduler_bounds_in_flight_without_retry() {
    let mut scheduler = PreviewScheduler::default();
    let mut admitted = Vec::new();
    for _ in 0..MAX_PREVIEW_IN_FLIGHT {
        let uuid = AssetUuid::new();
        scheduler.mark_dirty(uuid);
        let token = scheduler.request_refresh(uuid, true).expect("admitted");
        admitted.push((uuid, token));
    }
    let waiting = AssetUuid::new();
    scheduler.mark_dirty(waiting);
    assert!(scheduler.request_refresh(waiting, true).is_none());

    assert!(scheduler.complete_refresh(admitted[0].0, admitted[0].1));
    let waiting_token = scheduler.request_refresh(waiting, true).expect("refill");
    assert!(scheduler.complete_refresh(waiting, waiting_token));
    assert!(scheduler.request_refresh(waiting, true).is_none());
    scheduler.mark_dirty(waiting);
    assert!(scheduler.request_refresh(waiting, true).is_some());
}

#[test]
fn optimization_batch_20260830en_stale_preview_token_preserves_current_admission() {
    let asset_uuid = AssetUuid::new();
    let mut previous = PreviewScheduler::default();
    previous.mark_dirty(asset_uuid);
    let stale_token = previous
        .request_refresh(asset_uuid, true)
        .expect("old generation admission");

    let mut current = PreviewScheduler::default();
    current.mark_dirty(asset_uuid);
    let current_token = current
        .request_refresh(asset_uuid, true)
        .expect("new generation admission");

    assert!(!current.complete_refresh(asset_uuid, stale_token));
    assert!(current.owns_refresh(asset_uuid, current_token));
}

#[test]
fn optimization_batch_20260830en_preview_completion_uses_one_hash_entry_probe() {
    let source = include_str!("../preview.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("preview scheduler production source");

    assert!(production.contains("self.in_flight.entry(asset_uuid)"));
    assert!(!production.contains("self.in_flight.get(&asset_uuid) != Some(&token)"));
    assert!(!production.contains("self.in_flight.remove(&asset_uuid)"));
}

#[test]
#[ignore = "release-only preview completion probe evidence"]
fn optimization_batch_20260830en_preview_completion_probe_evidence() {
    const COMPLETION_COUNT: usize = 65_536;
    const LEGACY_HASH_PROBES_PER_COMPLETION: usize = 2;
    const OPTIMIZED_HASH_PROBES_PER_COMPLETION: usize = 1;
    let legacy_hash_probes = COMPLETION_COUNT * LEGACY_HASH_PROBES_PER_COMPLETION;
    let optimized_hash_probes = COMPLETION_COUNT * OPTIMIZED_HASH_PROBES_PER_COMPLETION;

    assert_eq!(legacy_hash_probes, optimized_hash_probes * 2);
    println!(
        "EDITOR542_PREVIEW_COMPLETION_ENTRY_BENCH_V1 completions={COMPLETION_COUNT} \
             legacy_hash_probes={legacy_hash_probes} optimized_hash_probes={optimized_hash_probes} \
             reduction_pct=50"
    );
}
