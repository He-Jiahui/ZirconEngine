use std::hint::black_box;
use std::time::Instant;

use super::*;

#[test]
fn optimization_batch_editor816_cycle_pending_capacity_preserves_cycle_verdict() {
    let edges = vec![edge("a", "b"), edge("b", "c")];
    let candidate = edge("c", "a");

    assert!(introduces_cycle(&edges, &candidate));
    assert_eq!(cycle_pending_capacity(edges.len()), edges.len() + 1);
    assert_eq!(cycle_pending_capacity(0), 1);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_editor816_cycle_pending_capacity_benchmark() {
    const EDGE_COUNT: usize = 4_096;
    const RUNS_PER_SAMPLE: usize = 4_096;
    const SAMPLE_PAIRS: usize = 17;

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_pending_growth(EDGE_COUNT, RUNS_PER_SAMPLE, false));
            optimized_samples.push(measure_pending_growth(EDGE_COUNT, RUNS_PER_SAMPLE, true));
        } else {
            optimized_samples.push(measure_pending_growth(EDGE_COUNT, RUNS_PER_SAMPLE, true));
            legacy_samples.push(measure_pending_growth(EDGE_COUNT, RUNS_PER_SAMPLE, false));
        }
    }

    let legacy_growth_events = growth_events(EDGE_COUNT + 1);
    let optimized_growth_events = 0;
    assert!(legacy_growth_events > optimized_growth_events);
    println!(
        "EDITOR816_GRAPH_CYCLE_PENDING_CAPACITY_BENCH_V1 edge_count={EDGE_COUNT} runs_per_sample={RUNS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_samples, 95),
        percentile(&optimized_samples, 95),
    );
}

fn edge(from: &str, to: &str) -> GraphEdgeView<String> {
    GraphEdgeView::new(
        GraphPortRef::output(from.to_owned(), "out"),
        GraphPortRef::input(to.to_owned(), "in"),
    )
}

fn measure_pending_growth(edge_count: usize, runs: usize, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..runs {
        let mut pending = if optimized {
            Vec::with_capacity(cycle_pending_capacity(edge_count))
        } else {
            Vec::new()
        };
        for index in 0..=edge_count {
            pending.push(index);
        }
        checksum = checksum.wrapping_add(pending.len());
        black_box(pending);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn growth_events(item_count: usize) -> usize {
    let mut capacity = 0usize;
    let mut events = 0usize;
    for length in 1..=item_count {
        if length > capacity {
            capacity = if capacity == 0 {
                4
            } else {
                capacity.saturating_mul(2)
            };
            events += 1;
        }
    }
    events
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
