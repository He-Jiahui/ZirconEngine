use std::hint::black_box;
use std::time::Instant;

use super::apply_external_effect_batch;

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const EFFECTS_PER_BUILD: usize = 256;

#[test]
fn optimization_batch_20260826fi_editor150_capacity_preserves_external_effect_order() {
    let effects = (0..EFFECTS_PER_BUILD).collect::<Vec<_>>();

    let affected = apply_external_effect_batch(&effects, |effect| {
        Ok::<_, &'static str>(effect.saturating_mul(2))
    })
    .expect("effect batch should apply");

    assert_eq!(affected.len(), EFFECTS_PER_BUILD);
    assert!(affected.capacity() >= EFFECTS_PER_BUILD);
    assert_eq!(affected[0], 0);
    assert_eq!(affected[EFFECTS_PER_BUILD - 1], (EFFECTS_PER_BUILD - 1) * 2);

    let source = include_str!("../navigation.rs");
    assert_eq!(
        source
            .matches("apply_external_effect_batch(&replay.external_effects")
            .count(),
        2
    );
}

#[test]
fn optimization_batch_20260826fi_editor150_external_effect_batch_stops_at_first_error() {
    let effects = [0, 1, 2, 3, 4, 5];
    let mut visited = Vec::new();

    let result = apply_external_effect_batch(&effects, |effect| {
        visited.push(*effect);
        (*effect != 3).then_some(*effect).ok_or("effect failed")
    });

    assert_eq!(result, Err("effect failed"));
    assert_eq!(visited, [0, 1, 2, 3]);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fi_editor150_external_effect_asset_capacity_bench() {
    let effects = (0..EFFECTS_PER_BUILD).collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&effects, false));
            optimized_samples.push(measure(&effects, true));
        } else {
            optimized_samples.push(measure(&effects, true));
            legacy_samples.push(measure(&effects, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR150_EXTERNAL_EFFECT_ASSET_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} effects_per_build={EFFECTS_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(effects: &[usize], reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let affected = if reserve {
            apply_external_effect_batch(effects, |effect| Ok::<_, ()>(black_box(*effect)))
                .expect("infallible effect batch")
        } else {
            let mut affected = Vec::new();
            for effect in effects {
                affected.push(black_box(*effect));
            }
            affected
        };
        checksum ^= black_box(affected.len() ^ affected.capacity());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
