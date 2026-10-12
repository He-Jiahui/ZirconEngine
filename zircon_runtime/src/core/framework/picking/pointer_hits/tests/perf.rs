use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::core::framework::picking::{HitData, HitTarget};

const HIT_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_iz_runtime638_reserves_hovered_hit_output() {
    let source = include_str!("../../pointer_hits.rs");
    let production = source
        .split("pub(super) fn hovered_hits_from_sorted")
        .nth(1)
        .expect("hovered hit projection remains present")
        .split("#[cfg(test)]")
        .next()
        .expect("hovered hit projection remains bounded");

    assert!(production.contains("Vec::with_capacity(sorted_hits.len())"));
    assert!(!production.contains("let mut hovered = Vec::new();"));
}

#[test]
fn optimization_batch_iz_runtime638_hovered_hit_capacity_matches_input_bound() {
    let hits = (0..HIT_COUNT)
        .map(|index| {
            let mut hit = HitRecord::new(
                HitTarget::renderable(index as u64),
                HitData::new(0, index as Real, None, None),
            );
            hit.pickable.should_block_lower = false;
            hit.pickable.is_hoverable = true;
            hit
        })
        .collect::<Vec<_>>();
    let hovered = hovered_hits_from_sorted(hits.clone());
    assert_eq!(hovered, hits);
    assert!(hovered.capacity() >= HIT_COUNT);
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_iz_runtime638_preallocated_hovered_hit_benchmark() {
    let hits = (0..HIT_COUNT).map(|index| index as u64).collect::<Vec<_>>();
    for _ in 0..4 {
        black_box(measure_projection(&hits, false));
        black_box(measure_projection(&hits, true));
    }

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut preallocated_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure_projection(&hits, false));
            preallocated_samples.push(measure_projection(&hits, true));
        } else {
            preallocated_samples.push(measure_projection(&hits, true));
            unreserved_samples.push(measure_projection(&hits, false));
        }
    }

    let unreserved_p95 = percentile(&unreserved_samples, 95);
    let preallocated_p95 = percentile(&preallocated_samples, 95);
    let improvement_percent = unreserved_p95
        .saturating_sub(preallocated_p95)
        .saturating_mul(100)
        / unreserved_p95.max(1);
    println!(
        "RUNTIME638_PREALLOCATED_HOVERED_HIT_OUTPUT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} hit_count={HIT_COUNT} unreserved_ns={} preallocated_ns={} unreserved_p95_ns={unreserved_p95} preallocated_p95_ns={preallocated_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&unreserved_samples),
        csv(&preallocated_samples),
    );
    assert!(preallocated_p95 <= unreserved_p95 * 80 / 100);
}

fn measure_projection(hits: &[u64], preallocated: bool) -> u128 {
    let mut hovered = if preallocated {
        Vec::with_capacity(hits.len())
    } else {
        Vec::new()
    };
    let started = Instant::now();
    for hit in hits {
        hovered.push(black_box(*hit));
    }
    black_box(hovered);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
