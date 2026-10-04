use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::core::framework::picking::HitData;

use super::*;

#[test]
fn runtime47_batch_hover_membership_tracks_mutation_and_first_hit() {
    let pointer = PointerId::new(7);
    let first = HitTarget::renderable(10);
    let second = HitTarget::scene_gizmo(20);
    let mut hover = PickingHoverMap::new(
        pointer,
        vec![hit(first, 0.1), hit(first, 0.2), hit(second, 0.3)],
    );

    assert!(hover.is_hovered(pointer, first));
    assert!(hover.is_hovered(pointer, second));
    assert_eq!(
        hover.hit(pointer, first).map(|hit| hit.hit.depth),
        Some(0.1)
    );
    assert_eq!(hover.get(pointer).len(), 3);

    hover.set_pointer_hits(pointer, vec![hit(second, 0.4)]);
    assert!(!hover.is_hovered(pointer, first));
    assert!(hover.is_hovered(pointer, second));
    hover.remove_pointer(pointer);
    assert!(!hover.is_hovered(pointer, second));
    assert!(hover.is_empty());
}

#[test]
fn runtime47_batch_hover_membership_uses_generation_index() {
    let source = include_str!("../hover_map.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("hover map implementation");

    assert!(production.contains("struct PointerHoverState"));
    assert!(production.contains("targets: HashSet<HitTarget>"));
    assert!(production.contains("state.targets.contains(&target)"));
    assert!(production.contains("Arc<BTreeMap<PointerId, PointerHoverState>>"));
}

#[test]
#[ignore = "managed release evidence"]
fn runtime47_batch_hover_membership_evidence() {
    const HIT_COUNT: usize = 2_048;
    const LOOKUP_COUNT: usize = 10_000;
    const SAMPLE_PAIRS: usize = 11;
    const TARGET: Duration = Duration::from_millis(50);

    let pointer = PointerId::new(1);
    let hits = (0..HIT_COUNT as u64)
        .map(|owner| hit(HitTarget::renderable(owner), owner as f32))
        .collect::<Vec<_>>();
    let hover = PickingHoverMap::new(pointer, hits);
    let tail = HitTarget::renderable(HIT_COUNT as u64 - 1);
    let missing = HitTarget::renderable(u64::MAX);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        let measure_legacy =
            || measure_ns(|| legacy_hover_checksum(&hover, pointer, tail, missing, LOOKUP_COUNT));
        let measure_optimized =
            || measure_ns(|| indexed_hover_checksum(&hover, pointer, tail, missing, LOOKUP_COUNT));
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95 = nearest_rank(&legacy_samples, 95);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    let comparisons_before = HIT_COUNT * LOOKUP_COUNT;
    let probes_after = LOOKUP_COUNT;
    let lookup_work_reduction_percent =
        (1.0 - probes_after as f64 / comparisons_before as f64) * 100.0;

    assert!(
        optimized_p95 <= TARGET.as_nanos(),
        "optimized_p95_ns={optimized_p95} target_ns={}",
        TARGET.as_nanos()
    );
    assert!(
        optimized_p95.saturating_mul(2) <= legacy_p95,
        "optimized_p95_ns={optimized_p95} legacy_p95_ns={legacy_p95}"
    );
    println!(
        "RUNTIME47_HOVER_MEMBERSHIP_BENCH_V1 hits={} lookups={} comparisons_before={} probes_after={} lookup_work_reduction_percent={:.4} legacy_p95_ns={} optimized_p95_ns={} target_ns={}",
        HIT_COUNT,
        LOOKUP_COUNT,
        comparisons_before,
        probes_after,
        lookup_work_reduction_percent,
        legacy_p95,
        optimized_p95,
        TARGET.as_nanos()
    );
}

fn legacy_hover_checksum(
    hover: &PickingHoverMap,
    pointer: PointerId,
    tail: HitTarget,
    missing: HitTarget,
    lookups: usize,
) -> u64 {
    let mut checksum = 0_u64;
    for lookup in 0..lookups {
        let target = black_box(if lookup % 2 == 0 { tail } else { missing });
        checksum += hover.get(pointer).iter().any(|hit| hit.target == target) as u64;
    }
    black_box(checksum)
}

fn indexed_hover_checksum(
    hover: &PickingHoverMap,
    pointer: PointerId,
    tail: HitTarget,
    missing: HitTarget,
    lookups: usize,
) -> u64 {
    let mut checksum = 0_u64;
    for lookup in 0..lookups {
        let target = black_box(if lookup % 2 == 0 { tail } else { missing });
        checksum += hover.is_hovered(pointer, target) as u64;
    }
    black_box(checksum)
}

fn measure_ns(measure: impl FnOnce() -> u64) -> u128 {
    let started = Instant::now();
    black_box(measure());
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn hit(target: HitTarget, depth: f32) -> HitRecord {
    HitRecord::new(target, HitData::new(0, depth, None, None))
}
