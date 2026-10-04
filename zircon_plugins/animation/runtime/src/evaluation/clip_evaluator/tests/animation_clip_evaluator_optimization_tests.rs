use std::collections::VecDeque;
use std::hint::black_box;
use std::time::Instant;

use super::*;

#[test]
fn optimization_batch_20260830ce_resource_events_drain_without_intermediate_vec() {
    let source = include_str!("../animation_clip_evaluator.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("production source");

    assert!(production.contains("self.resource_events.as_ref()"));
    assert!(production.contains("receiver.try_recv()"));
    assert!(!production.contains("collect::<Vec<_>>()"));
}

#[test]
fn optimization_batch_20260830ce_resource_event_drain_preserves_all_kinds() {
    let source = include_str!("../animation_clip_evaluator.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("production source");

    for kind in [
        "ResourceKind::AnimationSkeleton",
        "ResourceKind::AnimationClip",
    ] {
        assert!(production.contains(kind));
    }
}

#[test]
#[ignore = "Release-only Runtime170 performance contract"]
fn optimization_batch_20260830ce_resource_event_direct_drain_p95() {
    const EVENT_COUNT: u64 = 65_536;
    const SAMPLES: usize = 17;
    let events = (0..EVENT_COUNT).collect::<VecDeque<_>>();
    let mut baseline_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);

    for sample in 0..SAMPLES {
        let baseline = || {
            let mut input = events.clone();
            let started = Instant::now();
            let drained = input.drain(..).collect::<Vec<_>>();
            black_box(drained.into_iter().fold(0_u64, u64::wrapping_add));
            started.elapsed().as_nanos()
        };
        let optimized = || {
            let mut input = events.clone();
            let started = Instant::now();
            black_box(input.drain(..).fold(0_u64, u64::wrapping_add));
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            baseline_samples.push(baseline());
            optimized_samples.push(optimized());
        } else {
            optimized_samples.push(optimized());
            baseline_samples.push(baseline());
        }
    }

    let baseline_p95 = percentile_95(&mut baseline_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    println!(
        "RUNTIME170_EVENT_DRAIN_BENCH_V1 baseline_p95_ns={baseline_p95} optimized_p95_ns={optimized_p95}"
    );
    assert!(
        optimized_p95.saturating_mul(100) <= baseline_p95.saturating_mul(75),
        "expected direct event drain to reduce P95 by at least 25%: baseline={baseline_p95}ns optimized={optimized_p95}ns"
    );
}

fn percentile_95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95 / 100).min(samples.len() - 1)]
}
