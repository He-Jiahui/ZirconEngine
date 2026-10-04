use std::hint::black_box;
use std::time::Duration;

use super::*;

#[test]
fn render_io_frontier_recalculates_priority_when_a_waiter_leaves() {
    let mut frontier = RenderArtifactIoFrontier::<u8>::new();
    let low = RenderArtifactIoDemandKey::new(RenderArtifactIoPriority::LOW, None, 1);
    let critical = RenderArtifactIoDemandKey::new(RenderArtifactIoPriority::CRITICAL, None, 2);
    let normal = RenderArtifactIoDemandKey::new(RenderArtifactIoPriority::NORMAL, None, 3);
    frontier.add_waiter(1, low);
    frontier.enqueue(1, 1);
    frontier.add_waiter(2, normal);
    frontier.enqueue(2, 2);
    frontier.add_waiter(1, critical);

    frontier.remove_waiter(&1, critical);

    assert_eq!(frontier.pop_highest().map(|(_, key)| key), Some(2));
    assert_eq!(frontier.pop_highest().map(|(_, key)| key), Some(1));
}

#[test]
fn render_io_frontier_orders_equal_priority_by_deadline_then_fifo() {
    let mut frontier = RenderArtifactIoFrontier::<u8>::new();
    let now = Instant::now();
    for (key, deadline, sequence) in [
        (1, Some(now + Duration::from_secs(2)), 1),
        (2, Some(now + Duration::from_secs(1)), 2),
        (3, None, 3),
        (4, None, 4),
    ] {
        frontier.add_waiter(
            key,
            RenderArtifactIoDemandKey::new(
                RenderArtifactIoPriority::NORMAL,
                deadline,
                u64::from(key),
            ),
        );
        frontier.enqueue(key, sequence);
    }

    assert_eq!(frontier.pop_highest().map(|(_, key)| key), Some(2));
    assert_eq!(frontier.pop_highest().map(|(_, key)| key), Some(1));
    assert_eq!(frontier.pop_highest().map(|(_, key)| key), Some(3));
    assert_eq!(frontier.pop_highest().map(|(_, key)| key), Some(4));
}

#[test]
fn optimization_batch_hb_runtime583_frontier_pop_moves_owned_key() {
    let key = "resource/".repeat(256);
    let mut frontier = RenderArtifactIoFrontier::<String>::new();
    frontier.add_waiter(
        key.clone(),
        RenderArtifactIoDemandKey::new(RenderArtifactIoPriority::NORMAL, None, 1),
    );
    frontier.enqueue(key.clone(), 1);

    assert_eq!(frontier.pop_highest().map(|(_, key)| key), Some(key));
    assert_eq!(frontier.queued_len(), 0);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_hb_runtime583_frontier_owned_pop_p95() {
    const SAMPLE_PAIRS: usize = 21;
    const ENTRIES: usize = 512;
    const KEY_SEGMENTS: usize = 512;
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let legacy_frontier = benchmark_frontier(ENTRIES, KEY_SEGMENTS);
        let optimized_frontier = benchmark_frontier(ENTRIES, KEY_SEGMENTS);
        if pair % 2 == 0 {
            legacy.push(measure(legacy_frontier, false));
            optimized.push(measure(optimized_frontier, true));
        } else {
            optimized.push(measure(optimized_frontier, true));
            legacy.push(measure(legacy_frontier, false));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME583_FRONTIER_OWNED_POP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} entries={ENTRIES} \
key_segments={KEY_SEGMENTS} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(75),
        "owned frontier pop must improve long-key P95 by at least 25%"
    );
}

fn benchmark_frontier(entries: usize, key_segments: usize) -> RenderArtifactIoFrontier<String> {
    let mut frontier = RenderArtifactIoFrontier::new();
    let prefix = "resource-segment/".repeat(key_segments);
    for index in 0..entries {
        let key = format!("{prefix}{index}");
        frontier.add_waiter(
            key.clone(),
            RenderArtifactIoDemandKey::new(RenderArtifactIoPriority::NORMAL, None, index as u64),
        );
        frontier.enqueue(key, index as u64);
    }
    frontier
}

fn measure(mut frontier: RenderArtifactIoFrontier<String>, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut bytes = 0_usize;
    while !frontier.ordered.is_empty() {
        let popped = if optimized {
            frontier.pop_highest()
        } else {
            pop_highest_legacy(&mut frontier)
        }
        .expect("fixture frontier should remain populated");
        bytes ^= black_box(popped.1.len());
    }
    black_box(bytes);
    started.elapsed().as_nanos().max(1)
}

fn pop_highest_legacy(
    frontier: &mut RenderArtifactIoFrontier<String>,
) -> Option<(RenderArtifactIoFrontierKey, String)> {
    let (&frontier_key, key) = frontier.ordered.iter().next_back()?;
    let key = key.clone();
    frontier.ordered.remove(&frontier_key);
    frontier.queued.remove(&key);
    Some((frontier_key, key))
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
