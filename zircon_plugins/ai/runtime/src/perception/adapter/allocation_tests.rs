use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use zircon_runtime::core::framework::ai::AiHearingStimulusEvent;
use zircon_runtime::core::framework::scene::EntityId;
use zircon_runtime::core::math::{Real, Vec3};

use super::HearingStimulusAdapter;

const BENCHMARK_RECEIVER_COUNT: usize = 256;
const BENCHMARK_ITERATIONS: usize = 64;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

type Receiver = (EntityId, Vec3, Real, Real);

#[test]
fn hearing_adapter_defers_empty_work_and_reuses_receiver_index() {
    let source = include_str!("../adapter.rs");
    let enqueue = function_body(
        source,
        "pub(crate) fn enqueue(",
        "pub(crate) fn process_budgeted(",
    );
    assert!(enqueue.contains("let mut receiver_ids = None;"));
    assert!(enqueue.contains("receiver_ids.get_or_insert_with"));
    assert!(!enqueue.starts_with("let receiver_ids = receivers"));

    let process = function_body(
        source,
        "pub(crate) fn process_budgeted(",
        "pub(crate) struct HearingAdapterReport",
    );
    assert!(process.contains("if pair_limit == 0 || self.pending.is_empty()"));
    assert!(source.contains("receiver_index: HashMap<EntityId, Receiver>"));
    assert!(process.contains("self.receiver_index.clear();"));
    assert!(process.contains("self.receiver_index.extend("));
    assert!(!process.contains("collect::<BTreeMap"));
}

#[test]
fn empty_enqueue_and_processing_leave_adapter_idle() {
    let receivers = receivers();
    let mut adapter = HearingStimulusAdapter::default();

    adapter.enqueue(std::iter::empty::<AiHearingStimulusEvent>(), &receivers);
    let report = adapter.process_budgeted(&receivers, usize::MAX, || true, |_, _| {});

    assert_eq!(adapter.pending_event_count(), 0);
    assert_eq!(adapter.pending_receiver_snapshot_count(), 0);
    assert_eq!(report.processed_pairs, 0);
    assert_eq!(report.refreshed_stimuli, 0);
}

#[test]
fn reusable_index_preserves_receiver_snapshot_order() {
    let receivers = vec![receiver(1), receiver(2)];
    let current_receivers = vec![receiver(2), receiver(1)];
    let mut adapter = HearingStimulusAdapter::default();
    adapter.enqueue(
        [AiHearingStimulusEvent::sound_playback(99, Vec3::ZERO, 1.0)],
        &receivers,
    );
    let mut refreshed = Vec::new();

    let report = adapter.process_budgeted(
        &current_receivers,
        usize::MAX,
        || true,
        |receiver, _| refreshed.push(receiver),
    );

    assert_eq!(refreshed, vec![1, 2]);
    assert_eq!(report.processed_pairs, 2);
    assert_eq!(report.refreshed_stimuli, 2);
    assert_eq!(adapter.pending_event_count(), 0);
}

#[test]
#[ignore = "release-only performance evidence"]
fn lazy_empty_hearing_enqueue_release_benchmark_evidence() {
    let receivers = receivers();
    let mut adapter = HearingStimulusAdapter::default();
    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || {
            for _ in 0..BENCHMARK_ITERATIONS {
                let receiver_ids = receivers
                    .iter()
                    .map(|receiver| receiver.0)
                    .collect::<Arc<[_]>>();
                black_box(receiver_ids);
            }
        },
        || {
            for _ in 0..BENCHMARK_ITERATIONS {
                adapter.enqueue(std::iter::empty::<AiHearingStimulusEvent>(), &receivers);
            }
        },
    );
    let metrics = metrics(&legacy_samples, &optimized_samples);
    println!(
        "PERF_RESULT plugins15_lazy_empty_hearing_enqueue receivers={BENCHMARK_RECEIVER_COUNT} iterations={BENCHMARK_ITERATIONS} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_receiver_snapshot_allocations_per_iteration=1 optimized_receiver_snapshot_allocations_per_iteration=0 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={} optimized_ns={}",
        metrics.legacy_p50,
        metrics.legacy_p95,
        metrics.optimized_p50,
        metrics.optimized_p95,
        metrics.legacy_ns,
        metrics.optimized_ns,
    );
    assert!(
        metrics.optimized_p95.saturating_mul(5) <= metrics.legacy_p95,
        "empty enqueue P95 must be at most 20% of eager receiver snapshot P95"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn empty_hearing_processing_early_exit_release_benchmark_evidence() {
    let receivers = receivers();
    let mut adapter = HearingStimulusAdapter::default();
    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || {
            for _ in 0..BENCHMARK_ITERATIONS {
                let receiver_index = receivers
                    .iter()
                    .copied()
                    .map(|receiver| (receiver.0, receiver))
                    .collect::<BTreeMap<_, _>>();
                black_box(receiver_index);
            }
        },
        || {
            for _ in 0..BENCHMARK_ITERATIONS {
                black_box(adapter.process_budgeted(&receivers, usize::MAX, || true, |_, _| {}));
            }
        },
    );
    let metrics = metrics(&legacy_samples, &optimized_samples);
    println!(
        "PERF_RESULT plugins15_empty_hearing_processing_early_exit receivers={BENCHMARK_RECEIVER_COUNT} iterations={BENCHMARK_ITERATIONS} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_receiver_index_builds_per_iteration=1 optimized_receiver_index_builds_per_iteration=0 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={} optimized_ns={}",
        metrics.legacy_p50,
        metrics.legacy_p95,
        metrics.optimized_p50,
        metrics.optimized_p95,
        metrics.legacy_ns,
        metrics.optimized_ns,
    );
    assert!(
        metrics.optimized_p95.saturating_mul(5) <= metrics.legacy_p95,
        "empty processing P95 must be at most 20% of eager receiver index P95"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn reusable_hearing_receiver_index_release_benchmark_evidence() {
    let receivers = receivers();
    let mut receiver_index = HashMap::with_capacity(receivers.len());
    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || {
            for _ in 0..BENCHMARK_ITERATIONS {
                let receiver_index = receivers
                    .iter()
                    .copied()
                    .map(|receiver| (receiver.0, receiver))
                    .collect::<BTreeMap<_, _>>();
                black_box(receiver_index.get(&1));
            }
        },
        || {
            for _ in 0..BENCHMARK_ITERATIONS {
                receiver_index.clear();
                receiver_index.extend(
                    receivers
                        .iter()
                        .copied()
                        .map(|receiver| (receiver.0, receiver)),
                );
                black_box(receiver_index.get(&1));
            }
        },
    );
    let metrics = metrics(&legacy_samples, &optimized_samples);
    println!(
        "PERF_RESULT plugins15_reusable_hearing_receiver_index receivers={BENCHMARK_RECEIVER_COUNT} iterations={BENCHMARK_ITERATIONS} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_index_allocations_per_iteration=1 optimized_index_allocations_per_iteration=0 legacy_lookup_structure=btree optimized_lookup_structure=hash legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={} optimized_ns={}",
        metrics.legacy_p50,
        metrics.legacy_p95,
        metrics.optimized_p50,
        metrics.optimized_p95,
        metrics.legacy_ns,
        metrics.optimized_ns,
    );
    assert!(
        metrics.optimized_p95.saturating_mul(2) <= metrics.legacy_p95,
        "reused hash index P95 must be at most 50% of rebuilt BTreeMap P95"
    );
}

fn receivers() -> Vec<Receiver> {
    (1..=BENCHMARK_RECEIVER_COUNT as EntityId)
        .map(receiver)
        .collect()
}

fn receiver(entity: EntityId) -> Receiver {
    (entity, Vec3::ZERO, 100.0, 5.0)
}

fn function_body<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start = source.find(start).expect("function start exists");
    let end = source[start..]
        .find(end)
        .map(|offset| start + offset)
        .expect("function end exists");
    &source[start..end]
}

struct BenchmarkMetrics {
    legacy_p50: u128,
    legacy_p95: u128,
    optimized_p50: u128,
    optimized_p95: u128,
    legacy_ns: String,
    optimized_ns: String,
}

fn metrics(legacy_samples: &[u128], optimized_samples: &[u128]) -> BenchmarkMetrics {
    BenchmarkMetrics {
        legacy_p50: percentile(legacy_samples, 50),
        legacy_p95: percentile(legacy_samples, 95),
        optimized_p50: percentile(optimized_samples, 50),
        optimized_p95: percentile(optimized_samples, 95),
        legacy_ns: samples_csv(legacy_samples),
        optimized_ns: samples_csv(optimized_samples),
    }
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
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let index = (ordered.len() * percentile).div_ceil(100) - 1;
    ordered[index]
}

fn samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
