use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use super::{hover_diff, hover_diff_with_scratch};
use zircon_runtime_interface::ui::event_ui::UiNodeId;

#[test]
fn runtime200_hover_diff_preserves_route_order_without_set_allocation() {
    let shared = UiNodeId::new(1);
    let previous_leaf = UiNodeId::new(2);
    let current_leaf = UiNodeId::new(3);

    assert_eq!(
        hover_diff(&[current_leaf, shared], &[previous_leaf, shared]),
        (vec![current_leaf], vec![previous_leaf])
    );
    assert_eq!(hover_diff(&[shared], &[shared]), (Vec::new(), Vec::new()));
}

#[test]
fn runtime200_hover_diff_preserves_route_order_on_the_indexed_path() {
    let current = (1..=10).map(UiNodeId::new).collect::<Vec<_>>();
    let previous = (6..=15).map(UiNodeId::new).collect::<Vec<_>>();

    assert_eq!(
        hover_diff(&current, &previous),
        (
            (1..=5).map(UiNodeId::new).collect(),
            (11..=15).map(UiNodeId::new).collect(),
        )
    );
}

#[test]
fn runtime200_hover_diff_reuses_large_membership_scratch() {
    let current = (1..=128).map(UiNodeId::new).collect::<Vec<_>>();
    let previous = (129..=256).map(UiNodeId::new).collect::<Vec<_>>();
    let mut membership = HashSet::new();

    let first = hover_diff_with_scratch(&current, &previous, &mut membership);
    let capacity = membership.capacity();
    assert_eq!(first.0, current);
    assert_eq!(first.1, previous);
    assert_eq!(first.0.len(), 128);
    assert_eq!(first.1.len(), 128);
    assert!(capacity >= 128);

    let second = hover_diff_with_scratch(&previous, &current, &mut membership);
    assert_eq!(second.0, previous);
    assert_eq!(second.1, current);
    assert_eq!(second.0.len(), 128);
    assert_eq!(second.1.len(), 128);
    assert_eq!(membership.capacity(), capacity);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime200_hover_diff_membership_scratch_benchmark() {
    const EVENTS_PER_SAMPLE: usize = 16_384;
    const SAMPLE_COUNT: usize = 11;
    const MARKER: &str = "RUNTIME200_HOVER_DIFF_MEMBERSHIP_SCRATCH_BENCH_V1";

    let current = (1..=128).map(UiNodeId::new).collect::<Vec<_>>();
    let previous = (129..=256).map(UiNodeId::new).collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let (legacy, optimized) = if sample % 2 == 0 {
            (measure_legacy(&current, &previous, EVENTS_PER_SAMPLE), {
                let mut membership = HashSet::new();
                hover_diff_with_scratch(&current, &previous, &mut membership);
                measure_optimized(&current, &previous, &mut membership, EVENTS_PER_SAMPLE)
            })
        } else {
            let mut membership = HashSet::new();
            hover_diff_with_scratch(&current, &previous, &mut membership);
            (
                measure_legacy(&current, &previous, EVENTS_PER_SAMPLE),
                measure_optimized(&current, &previous, &mut membership, EVENTS_PER_SAMPLE),
            )
        };
        legacy_samples.push(legacy);
        optimized_samples.push(optimized);
    }

    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "{MARKER} samples={SAMPLE_COUNT} events_per_sample={EVENTS_PER_SAMPLE} \
             legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} \
             legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert!(optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(80));
}

fn measure_legacy(current: &[UiNodeId], previous: &[UiNodeId], events: usize) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..events {
        let (entered, left) = black_box(hover_diff(current, previous));
        checksum = checksum
            .wrapping_add(entered.len())
            .wrapping_add(left.len());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(
    current: &[UiNodeId],
    previous: &[UiNodeId],
    membership: &mut HashSet<UiNodeId>,
    events: usize,
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..events {
        let (entered, left) = black_box(hover_diff_with_scratch(current, previous, membership));
        checksum = checksum
            .wrapping_add(entered.len())
            .wrapping_add(left.len());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
