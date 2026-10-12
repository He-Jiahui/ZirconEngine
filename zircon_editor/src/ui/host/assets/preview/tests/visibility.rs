use std::collections::{HashMap, HashSet};
use std::hint::black_box;
use std::mem::size_of;
use std::time::Instant;

use zircon_runtime::asset::AssetUuid;

use super::{PreviewJobToken, PreviewScheduler, MAX_PREVIEW_IN_FLIGHT};

// Mirrors the request path before removal of the write-only resident visible set.
#[derive(Default)]
struct LegacyPreviewScheduler {
    dirty: HashSet<AssetUuid>,
    visible: HashSet<AssetUuid>,
    in_flight: HashMap<AssetUuid, PreviewJobToken>,
}

impl LegacyPreviewScheduler {
    fn request_refresh(&mut self, asset_uuid: AssetUuid, visible: bool) -> Option<PreviewJobToken> {
        if visible {
            self.visible.insert(asset_uuid);
        } else {
            self.visible.remove(&asset_uuid);
        }

        if !visible
            || self.in_flight.contains_key(&asset_uuid)
            || self.in_flight.len() >= MAX_PREVIEW_IN_FLIGHT
            || !self.dirty.remove(&asset_uuid)
        {
            return None;
        }
        let token = PreviewJobToken::next();
        self.in_flight.insert(asset_uuid, token);
        Some(token)
    }
}

#[test]
fn editor04_preview_scheduler_invisible_request_preserves_dirty_and_current_token() {
    let asset_uuid = AssetUuid::from_stable_label("editor04-invisible-preview");
    let mut scheduler = PreviewScheduler::default();
    scheduler.mark_dirty(asset_uuid);

    assert!(scheduler.request_refresh(asset_uuid, false).is_none());
    assert!(scheduler.dirty.contains(&asset_uuid));
    let token = scheduler
        .request_refresh(asset_uuid, true)
        .expect("visible dirty asset is admitted");
    assert!(!scheduler.dirty.contains(&asset_uuid));
    assert!(scheduler.request_refresh(asset_uuid, false).is_none());
    assert!(scheduler.owns_refresh(asset_uuid, token));
    assert!(scheduler.complete_refresh(asset_uuid, token));
    assert!(scheduler.request_refresh(asset_uuid, true).is_none());
}

#[test]
fn editor04_preview_scheduler_has_no_resident_visible_set() {
    assert_eq!(
        size_of::<LegacyPreviewScheduler>(),
        size_of::<PreviewScheduler>() + size_of::<HashSet<AssetUuid>>()
    );
}

#[test]
#[ignore = "Windows Release Editor04 100k preview request sweep evidence"]
fn editor04_preview_scheduler_visible_set_100k_release_benchmark() {
    const ASSET_COUNT: usize = 100_000;
    const WARMUP_COUNT: usize = 5;
    const SAMPLE_COUNT: usize = 31;

    assert!(
        !cfg!(debug_assertions),
        "run this comparison in Release mode"
    );
    let uuids = (0..ASSET_COUNT)
        .map(|index| AssetUuid::from_stable_label(&format!("editor04-preview-{index:06}")))
        .collect::<Vec<_>>();
    let dirty = uuids.iter().copied().collect::<HashSet<_>>();
    assert_eq!(dirty.len(), ASSET_COUNT);

    for warmup in 0..WARMUP_COUNT {
        let (mut legacy, mut current) = scheduler_pair(&dirty);
        if warmup % 2 == 0 {
            black_box(measure_legacy(&mut legacy, &uuids));
            black_box(measure_current(&mut current, &uuids));
        } else {
            black_box(measure_current(&mut current, &uuids));
            black_box(measure_legacy(&mut legacy, &uuids));
        }
        assert_equivalent_outcomes(&legacy, &current, ASSET_COUNT);
    }

    let mut raw_pairs_ns = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        let (mut legacy, mut current) = scheduler_pair(&dirty);
        let pair = if sample % 2 == 0 {
            (
                measure_legacy(&mut legacy, &uuids),
                measure_current(&mut current, &uuids),
            )
        } else {
            let current_ns = measure_current(&mut current, &uuids);
            let legacy_ns = measure_legacy(&mut legacy, &uuids);
            (legacy_ns, current_ns)
        };
        assert_equivalent_outcomes(&legacy, &current, ASSET_COUNT);
        raw_pairs_ns.push(pair);
    }

    let legacy_samples = raw_pairs_ns.iter().map(|pair| pair.0).collect::<Vec<_>>();
    let current_samples = raw_pairs_ns.iter().map(|pair| pair.1).collect::<Vec<_>>();
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let legacy_p99 = percentile(&legacy_samples, 99);
    let current_p50 = percentile(&current_samples, 50);
    let current_p95 = percentile(&current_samples, 95);
    let current_p99 = percentile(&current_samples, 99);
    println!(
        "PERF_RESULT EDITOR04_PREVIEW_VISIBLE_SET_100K_BENCH_V1 assets={ASSET_COUNT} warmups={WARMUP_COUNT} samples={SAMPLE_COUNT} order=legacy_first_even_pair legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} current_p50_ns={current_p50} current_p95_ns={current_p95} current_p99_ns={current_p99} old_visible_count={ASSET_COUNT} current_resident_visible_set=false in_flight_cap={MAX_PREVIEW_IN_FLIGHT} raw_pairs_legacy_current_ns={raw_pairs_ns:?} threshold_p95_ratio=80% os={} arch={} package_version={}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("CARGO_PKG_VERSION")
    );
    assert!(
        current_p95.saturating_mul(100) <= legacy_p95.saturating_mul(80),
        "preview request sweep p95 {current_p95}ns exceeded 80% of legacy {legacy_p95}ns"
    );
}

fn scheduler_pair(dirty: &HashSet<AssetUuid>) -> (LegacyPreviewScheduler, PreviewScheduler) {
    let in_flight = HashMap::new();
    let legacy = LegacyPreviewScheduler {
        dirty: dirty.clone(),
        visible: HashSet::new(),
        in_flight: in_flight.clone(),
    };
    let current = PreviewScheduler {
        dirty: dirty.clone(),
        in_flight,
    };
    assert_eq!(legacy.dirty, current.dirty);
    (legacy, current)
}

fn measure_legacy(scheduler: &mut LegacyPreviewScheduler, uuids: &[AssetUuid]) -> u128 {
    let started = Instant::now();
    for &uuid in uuids {
        let _ = scheduler.request_refresh(uuid, true);
    }
    let elapsed = started.elapsed().as_nanos().max(1);
    black_box(scheduler);
    elapsed
}

fn measure_current(scheduler: &mut PreviewScheduler, uuids: &[AssetUuid]) -> u128 {
    let started = Instant::now();
    for &uuid in uuids {
        let _ = scheduler.request_refresh(uuid, true);
    }
    let elapsed = started.elapsed().as_nanos().max(1);
    black_box(scheduler);
    elapsed
}

fn assert_equivalent_outcomes(
    legacy: &LegacyPreviewScheduler,
    current: &PreviewScheduler,
    asset_count: usize,
) {
    assert_eq!(legacy.visible.len(), asset_count);
    assert_eq!(legacy.dirty, current.dirty);
    assert_eq!(current.dirty.len(), asset_count - MAX_PREVIEW_IN_FLIGHT);
    assert_eq!(legacy.in_flight.len(), MAX_PREVIEW_IN_FLIGHT);
    assert_eq!(current.in_flight.len(), MAX_PREVIEW_IN_FLIGHT);
    assert_eq!(
        legacy.in_flight.keys().collect::<HashSet<_>>(),
        current.in_flight.keys().collect::<HashSet<_>>()
    );
    for (&uuid, &token) in &current.in_flight {
        assert!(current.owns_refresh(uuid, token));
    }
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100) - 1]
}
