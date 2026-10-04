use std::hint::black_box;
use std::time::Instant;

use super::Events;

const SAMPLE_PAIRS: usize = 17;
const PREVIOUS_CAPACITY: usize = 65_536;

#[test]
fn runtime60_batch_specialized_batch_extend_preserves_order_and_count() {
    let mut events = Events::default();

    assert_eq!(events.send_batch(10_u32..14), 4);
    events.update();

    assert_eq!(events.iter().copied().collect::<Vec<_>>(), [10, 11, 12, 13]);
}

#[test]
fn runtime60_batch_specialized_batch_extend_accepts_non_exact_iterators() {
    let mut events = Events::default();

    assert_eq!(
        events.send_batch((0_u32..8).filter(|value| value % 2 == 0)),
        4
    );
    events.update();

    assert_eq!(events.iter().copied().collect::<Vec<_>>(), [0, 2, 4, 6]);
}

#[test]
fn runtime60_batch_empty_batch_does_not_raise_the_high_water_mark() {
    let mut events = Events::<u32>::default();

    assert_eq!(events.send_batch(std::iter::empty()), 0);

    let metrics = events.capacity_metrics();
    assert_eq!(metrics.next_len, 0);
    assert_eq!(metrics.high_water_len, 0);
}

#[test]
fn runtime60_event_queue_reserves_next_buffer_after_two_growing_bursts() {
    let mut events = Events::<u64>::default();

    assert_eq!(events.send_batch(0..64), 64);
    events.update();
    assert!(events.capacity_metrics().next_capacity >= 64);

    assert_eq!(events.send_batch(0..128), 128);
    events.update();
    let metrics = events.capacity_metrics();
    assert_eq!(metrics.current_len, 128);
    assert_eq!(metrics.next_len, 0);
    assert_eq!(metrics.high_water_len, 128);
    assert!(
        metrics.next_capacity >= metrics.high_water_len,
        "the empty next buffer must retain enough capacity for the observed burst"
    );

    let reserved_capacity = metrics.next_capacity;
    assert_eq!(events.send_batch(0..128), 128);
    assert_eq!(events.capacity_metrics().next_capacity, reserved_capacity);
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn runtime60_event_queue_high_water_reserve_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
            legacy_samples.push(measure_growing_burst(false));
            optimized_samples.push(measure_growing_burst(true));
        } else {
            optimized_samples.push(measure_growing_burst(true));
            legacy_samples.push(measure_growing_burst(false));
        }
    }
    assert!(legacy_samples.iter().all(|(_, grew)| *grew));
    assert!(optimized_samples.iter().all(|(_, grew)| !*grew));
    let legacy_raw = legacy_samples.iter().map(|(ns, _)| *ns).collect::<Vec<_>>();
    let optimized_raw = optimized_samples
        .iter()
        .map(|(ns, _)| *ns)
        .collect::<Vec<_>>();
    let legacy_p95_ns = nearest_rank(&legacy_raw, 95);
    let optimized_p95_ns = nearest_rank(&optimized_raw, 95);
    println!(
        "RUNTIME60_EVENT_QUEUE_HIGH_WATER_RESERVE_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} sample_pairs={SAMPLE_PAIRS} previous_capacity={PREVIOUS_CAPACITY} timed_burst_events={} send_path_growth=1->0 legacy_raw_ns={legacy_raw:?} optimized_raw_ns={optimized_raw:?}",
        PREVIOUS_CAPACITY * 2
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(95),
        "preallocated next-buffer send P95 should be at most 95% of legacy"
    );
}

fn measure_growing_burst(optimized: bool) -> (u64, bool) {
    let mut events = Events::<u64>::default();
    events.next = Vec::with_capacity(PREVIOUS_CAPACITY);
    events.high_water_len = events.next.capacity() * 2;
    if optimized {
        events.reserve_next_for_high_water();
    } else {
        let legacy_additional = events.high_water_len - events.next.capacity();
        events.next.reserve_exact(legacy_additional);
    }
    let capacity_before_send = events.next.capacity();
    let started = Instant::now();
    for value in 0..events.high_water_len {
        events.send(black_box(value as u64));
    }
    let elapsed_ns = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    black_box(&events.next);
    (elapsed_ns, events.next.capacity() > capacity_before_send)
}

fn nearest_rank(samples: &[u64], percentile: usize) -> u64 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

#[test]
fn event_queue_equality_ignores_reader_generation_metadata() {
    let mut first = Events::<u32>::default();
    let mut second = Events::<u32>::default();

    first.update();
    first.update();

    assert_eq!(first, second);

    first.send(5);
    second.send(5);

    assert_eq!(first, second);
}
