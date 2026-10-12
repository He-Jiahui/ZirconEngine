use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::ai::{
    AiBehaviorNodeDescriptor, AiBehaviorNodeKind, AiBehaviorTreeDescriptor,
};

use super::{compile_behavior_tree, BehaviorNodeSlot, CompiledBehaviorTree};

const BENCHMARK_NODE_COUNT: usize = 4_096;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn compiled_implementation_slots_preserve_first_occurrence_order_without_duplicates() {
    let tree = wide_tree(8);
    let expected = legacy_unique_implementation_slots(&tree);
    let compiled = tree.implementation_slots().collect::<Vec<_>>();

    assert_eq!(compiled, expected);
    assert_eq!(compiled.len(), 2, "selector and task slots stay unique");
}

#[test]
fn implementation_slot_iteration_reads_the_compiled_unique_index() {
    let source = include_str!("../compile.rs");
    let fields = source
        .split("pub struct CompiledBehaviorTree {")
        .nth(1)
        .and_then(|body| body.split("impl CompiledBehaviorTree").next())
        .expect("compiled tree fields");
    let implementation_slots = source
        .split("pub(crate) fn implementation_slots(")
        .nth(1)
        .and_then(|body| body.split("pub(crate) fn has_abort_observers").next())
        .expect("implementation_slots body");

    assert!(fields.contains("implementation_slots: Box<[BehaviorNodeSlot]>"));
    assert!(implementation_slots.contains("self.implementation_slots.iter().copied()"));
    assert!(!implementation_slots.contains("self.nodes"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn compiled_unique_implementation_slots_release_benchmark_evidence() {
    let tree = wide_tree(BENCHMARK_NODE_COUNT);
    let expected = legacy_unique_implementation_slots(&tree);
    assert_eq!(expected.len(), 2);
    assert_eq!(tree.implementation_slots().collect::<Vec<_>>(), expected);

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || implementation_slot_checksum(legacy_unique_implementation_slots(black_box(&tree))),
        || implementation_slot_checksum(compiled_unique_implementation_slots(black_box(&tree))),
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);

    println!(
        "PERF_RESULT plugins15_compiled_unique_implementation_slots nodes={BENCHMARK_NODE_COUNT} unique_slots={} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_node_slot_reads_per_sample={BENCHMARK_NODE_COUNT} optimized_compiled_slot_reads_per_sample={} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}",
        expected.len(),
        expected.len(),
    );
    assert!(
        optimized_p95 * 10 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be no more than 10% of legacy P95 {legacy_p95}ns"
    );
}

fn wide_tree(node_count: usize) -> CompiledBehaviorTree {
    assert!(node_count >= 2);
    let mut root = AiBehaviorNodeDescriptor::new("root", AiBehaviorNodeKind::Selector, "Root");
    for index in 1..node_count {
        root = root.with_child(format!("leaf_{index:04}"));
    }
    let mut descriptor = AiBehaviorTreeDescriptor::new("wide", "Wide", "root").with_node(root);
    for index in 1..node_count {
        descriptor = descriptor.with_node(AiBehaviorNodeDescriptor::new(
            format!("leaf_{index:04}"),
            AiBehaviorNodeKind::Task,
            format!("Leaf {index}"),
        ));
    }
    compile_behavior_tree(&descriptor).expect("valid wide tree")
}

fn legacy_unique_implementation_slots(tree: &CompiledBehaviorTree) -> Vec<BehaviorNodeSlot> {
    let mut slots = Vec::new();
    for node in tree.nodes() {
        let slot = node.implementation();
        if !slots.contains(&slot) {
            slots.push(slot);
        }
    }
    black_box(slots)
}

fn compiled_unique_implementation_slots(tree: &CompiledBehaviorTree) -> Vec<BehaviorNodeSlot> {
    let slots = tree.implementation_slots().collect::<Vec<_>>();
    black_box(slots)
}

fn implementation_slot_checksum(slots: Vec<BehaviorNodeSlot>) -> u64 {
    black_box(&slots);
    slots
        .into_iter()
        .map(|slot| u64::from(slot.raw()) + 1)
        .sum()
}

fn benchmark_paired_samples(
    mut legacy: impl FnMut() -> u64,
    mut optimized: impl FnMut() -> u64,
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

fn benchmark_sample(operation: &mut impl FnMut() -> u64) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn benchmark_samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    assert!(!sorted.is_empty());
    assert!((1..=100).contains(&percentile));
    let index = (sorted.len() * percentile).div_ceil(100) - 1;
    sorted[index]
}
