use std::{collections::BinaryHeap, hint::black_box, time::Instant};

use zircon_runtime::core::framework::net::{
    RpcDirection, RpcInvocationDescriptor, RpcPeerRole,
};

use super::{take_queued_invocations, QueuedRpcInvocation};

const RPC_QUEUE_BENCHMARK_DEPTH: usize = 100_000;
const RPC_QUEUE_BENCHMARK_DRAIN: usize = 64;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
#[ignore = "release-only performance evidence"]
fn rpc_priority_heap_release_benchmark_evidence() {
    let legacy_base = benchmark_invocations();
    let heap_base = legacy_base.iter().cloned().collect::<BinaryHeap<_>>();

    let mut legacy_equivalence = legacy_base.clone();
    let mut heap_equivalence = heap_base.clone();
    assert_eq!(
        legacy_take_queued_invocations(&mut legacy_equivalence, RPC_QUEUE_BENCHMARK_DRAIN)
            .iter()
            .map(|queued| queued.sequence)
            .collect::<Vec<_>>(),
        take_queued_invocations(&mut heap_equivalence, RPC_QUEUE_BENCHMARK_DRAIN)
            .iter()
            .map(|queued| queued.sequence)
            .collect::<Vec<_>>()
    );

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || {
            let mut queued = legacy_base.clone();
            legacy_take_queued_invocations(&mut queued, RPC_QUEUE_BENCHMARK_DRAIN)
        },
        || {
            let mut queued = heap_base.clone();
            take_queued_invocations(&mut queued, RPC_QUEUE_BENCHMARK_DRAIN)
        },
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_front_shifts = RPC_QUEUE_BENCHMARK_DEPTH - RPC_QUEUE_BENCHMARK_DRAIN;
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);

    println!(
        "PERF_RESULT plugins10_rpc_priority_heap depth={} drain={} samples={} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_full_queue_sorts=1 optimized_full_queue_sorts=0 legacy_front_shifts={} optimized_front_shifts=0 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={legacy_ns} optimized_ns={optimized_ns}",
        RPC_QUEUE_BENCHMARK_DEPTH,
        RPC_QUEUE_BENCHMARK_DRAIN,
        BENCHMARK_SAMPLE_COUNT,
        legacy_front_shifts,
        legacy_p50,
        legacy_p95,
        optimized_p50,
        optimized_p95
    );
    assert!(
        optimized_p95 * 5 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be no more than 20% of legacy P95 {legacy_p95}ns"
    );
}

fn benchmark_samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn benchmark_invocations() -> Vec<QueuedRpcInvocation> {
    let now = Instant::now();
    (0..RPC_QUEUE_BENCHMARK_DEPTH as u64)
        .map(|sequence| QueuedRpcInvocation {
            invocation: RpcInvocationDescriptor::new(
                "benchmark.rpc",
                RpcDirection::ServerToClient,
                Vec::new(),
            )
            .with_priority(((sequence * 17) % 256) as u8),
            caller: RpcPeerRole::Server,
            enqueued_at: now,
            sequence,
        })
        .collect()
}

fn benchmark_paired_samples<L, O>(
    mut legacy: impl FnMut() -> L,
    mut optimized: impl FnMut() -> O,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            legacy_samples.push(benchmark_sample(&mut legacy));
            optimized_samples.push(benchmark_sample(&mut optimized));
        } else {
            optimized_samples.push(benchmark_sample(&mut optimized));
            legacy_samples.push(benchmark_sample(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn benchmark_sample<T>(operation: &mut impl FnMut() -> T) -> u128 {
    let started = Instant::now();
    let result = black_box(operation());
    let elapsed = started.elapsed().as_nanos();
    black_box(&result);
    elapsed
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    assert!(!sorted.is_empty());
    assert!((1..=100).contains(&percentile));
    let index = (sorted.len() * percentile).div_ceil(100) - 1;
    sorted[index]
}

fn legacy_take_queued_invocations(
    queued: &mut Vec<QueuedRpcInvocation>,
    max_invocations: usize,
) -> Vec<QueuedRpcInvocation> {
    queued.sort_by(|left, right| {
        right
            .invocation
            .priority
            .cmp(&left.invocation.priority)
            .then_with(|| left.sequence.cmp(&right.sequence))
    });
    let take = max_invocations.min(queued.len());
    queued.drain(0..take).collect()
}
