use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::ai::{
    AiAgentTickReport, AiBehaviorDebugFrame, AiBehaviorDebugSnapshot, AiDecisionStatus,
    BtNodeResultEvent,
};
use zircon_runtime::core::framework::scene::WorldHandle;

use super::{AiBtNodeResultMirror, AiBtNodeResultMirrorApply};

const LOOKUP_NODE_COUNT: usize = 4_096;
const LOOKUP_ITERATIONS: usize = 65_536;
const PRUNE_AGENT_COUNT: usize = 2_048;
const PRUNE_RESULTS_PER_AGENT: usize = 4;
const PRUNE_ITERATIONS: usize = 64;
const BENCHMARK_SAMPLES: usize = 21;

#[test]
fn grouped_node_results_preserve_world_and_active_node_pruning_semantics() {
    let mut mirror = AiBtNodeResultMirror::default();
    mirror.begin_session(12);
    assert_eq!(
        mirror.apply_node_result(12, 1, node_result(7, 44, "search")),
        AiBtNodeResultMirrorApply::Applied
    );
    assert_eq!(
        mirror.apply_node_result(12, 2, node_result(7, 44, "move_to")),
        AiBtNodeResultMirrorApply::Applied
    );
    assert_eq!(
        mirror.apply_node_result(12, 3, node_result(7, 45, "wait")),
        AiBtNodeResultMirrorApply::Applied
    );
    assert_eq!(
        mirror.apply_node_result(12, 4, node_result(8, 44, "other_world")),
        AiBtNodeResultMirrorApply::Applied
    );

    assert_eq!(
        mirror.apply_debug_snapshot(
            12,
            1,
            AiBehaviorDebugSnapshot {
                world: WorldHandle::new(7),
                frames: vec![
                    debug_frame(7, 44, Some("move_to")),
                    debug_frame(7, 45, None)
                ],
            },
        ),
        AiBtNodeResultMirrorApply::Applied
    );

    assert!(mirror
        .node_result(&WorldHandle::new(7), 44, "search")
        .is_none());
    assert!(mirror
        .node_result(&WorldHandle::new(7), 44, "move_to")
        .is_some());
    assert!(mirror
        .node_result(&WorldHandle::new(7), 45, "wait")
        .is_none());
    assert!(mirror
        .node_result(&WorldHandle::new(8), 44, "other_world")
        .is_some());
}

#[test]
fn duplicate_agent_frames_preserve_each_reported_active_node() {
    let mut mirror = AiBtNodeResultMirror::default();
    mirror.begin_session(12);
    assert_eq!(
        mirror.apply_node_result(12, 1, node_result(7, 44, "search")),
        AiBtNodeResultMirrorApply::Applied
    );
    assert_eq!(
        mirror.apply_node_result(12, 2, node_result(7, 44, "move_to")),
        AiBtNodeResultMirrorApply::Applied
    );

    assert_eq!(
        mirror.apply_debug_snapshot(
            12,
            1,
            AiBehaviorDebugSnapshot {
                world: WorldHandle::new(7),
                frames: vec![
                    debug_frame(7, 44, Some("search")),
                    debug_frame(7, 44, Some("move_to")),
                ],
            },
        ),
        AiBtNodeResultMirrorApply::Applied
    );

    assert!(mirror
        .node_result(&WorldHandle::new(7), 44, "search")
        .is_some());
    assert!(mirror
        .node_result(&WorldHandle::new(7), 44, "move_to")
        .is_some());
}

#[test]
fn node_result_lookup_and_snapshot_pruning_borrow_node_ids() {
    let source = include_str!("../runtime_mirror.rs");
    let lookup = source
        .split("pub fn node_result(")
        .nth(1)
        .and_then(|body| body.split("\n    }").next())
        .expect("node_result body");
    let prune = source
        .split("let mut active_nodes")
        .nth(1)
        .and_then(|body| body.split("AiBtNodeResultMirrorApply::Applied").next())
        .expect("snapshot prune body");

    assert!(source.contains("BTreeMap<(u64, u64), BTreeMap<String, BtNodeResultEvent>>"));
    assert!(prune.contains("BTreeMap::<u64, BTreeSet<String>>::new()"));
    assert!(!lookup.contains("node_id.to_owned()"));
    assert!(!lookup.contains("node_id.to_string()"));
    assert!(prune.contains("active_nodes.get(entity)"));
    assert!(!prune.contains("node_id.clone()"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn borrowed_node_result_lookup_release_benchmark_evidence() {
    let world = WorldHandle::new(7);
    let mut mirror = AiBtNodeResultMirror::default();
    mirror.begin_session(12);
    let mut legacy = BTreeMap::new();
    for index in 0..LOOKUP_NODE_COUNT {
        let node_id = format!("node-{index:04}");
        let event = node_result(7, 44, &node_id);
        legacy.insert((7, 44, node_id), event.clone());
        assert_eq!(
            mirror.apply_node_result(12, index as u64 + 1, event),
            AiBtNodeResultMirrorApply::Applied
        );
    }
    let target = "node-4095";
    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || {
            let mut hits = 0_u64;
            for _ in 0..LOOKUP_ITERATIONS {
                hits += legacy.get(&(7, 44, black_box(target).to_owned())).is_some() as u64;
            }
            hits
        },
        || {
            let mut hits = 0_u64;
            for _ in 0..LOOKUP_ITERATIONS {
                hits += mirror.node_result(&world, 44, black_box(target)).is_some() as u64;
            }
            hits
        },
    );
    print_performance_result(
        "plugins15_borrowed_editor_node_result_lookup",
        &legacy_samples,
        &optimized_samples,
        format!(
            "nodes={LOOKUP_NODE_COUNT} iterations_per_sample={LOOKUP_ITERATIONS} legacy_node_id_allocations_per_sample={LOOKUP_ITERATIONS} optimized_node_id_allocations_per_sample=0"
        ),
        3,
        5,
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn grouped_snapshot_pruning_release_benchmark_evidence() {
    let (legacy_results, active_nodes, grouped_results) = prune_fixture();
    let expected = PRUNE_AGENT_COUNT;
    assert_eq!(
        legacy_prune_match_count(&legacy_results, &active_nodes),
        expected
    );
    assert_eq!(
        grouped_prune_match_count(&grouped_results, &active_nodes),
        expected
    );

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || {
            let mut matches = 0_u64;
            for _ in 0..PRUNE_ITERATIONS {
                matches +=
                    legacy_prune_match_count(black_box(&legacy_results), black_box(&active_nodes))
                        as u64;
            }
            matches
        },
        || {
            let mut matches = 0_u64;
            for _ in 0..PRUNE_ITERATIONS {
                matches += grouped_prune_match_count(
                    black_box(&grouped_results),
                    black_box(&active_nodes),
                ) as u64;
            }
            matches
        },
    );
    print_performance_result(
        "plugins15_grouped_editor_snapshot_pruning",
        &legacy_samples,
        &optimized_samples,
        format!(
            "agents={PRUNE_AGENT_COUNT} results_per_agent={PRUNE_RESULTS_PER_AGENT} iterations_per_sample={PRUNE_ITERATIONS} legacy_node_id_allocations_per_iteration={} optimized_node_id_allocations_per_iteration=0",
            PRUNE_AGENT_COUNT * PRUNE_RESULTS_PER_AGENT
        ),
        3,
        4,
    );
}

fn node_result(world: u64, entity: u64, node_id: &str) -> BtNodeResultEvent {
    BtNodeResultEvent {
        world: WorldHandle::new(world),
        entity,
        node_id: node_id.to_owned(),
        status: AiDecisionStatus::Running,
        diagnostic: None,
    }
}

fn debug_frame(world: u64, entity: u64, active_node: Option<&str>) -> AiBehaviorDebugFrame {
    AiBehaviorDebugFrame {
        report: AiAgentTickReport {
            world: WorldHandle::new(world),
            entity,
            status: AiDecisionStatus::Running,
            active_node: active_node.map(str::to_owned),
            diagnostic: None,
        },
        behavior_tree: None,
        blackboard: Vec::new(),
        perception: None,
        perception_debug: None,
    }
}

type LegacyResults = BTreeMap<(u64, u64, String), BtNodeResultEvent>;
type GroupedResults = BTreeMap<(u64, u64), BTreeMap<String, BtNodeResultEvent>>;
type ActiveNodes = BTreeMap<u64, String>;

fn prune_fixture() -> (LegacyResults, ActiveNodes, GroupedResults) {
    let mut legacy = BTreeMap::new();
    let mut active = BTreeMap::new();
    let mut grouped = BTreeMap::new();
    for entity in 0..PRUNE_AGENT_COUNT as u64 {
        let active_node = format!("node-{entity:04}-0");
        active.insert(entity, active_node);
        let nodes = grouped.entry((7, entity)).or_insert_with(BTreeMap::new);
        for node_index in 0..PRUNE_RESULTS_PER_AGENT {
            let node_id = format!("node-{entity:04}-{node_index}");
            let event = node_result(7, entity, &node_id);
            legacy.insert((7, entity, node_id.clone()), event.clone());
            nodes.insert(node_id, event);
        }
    }
    (legacy, active, grouped)
}

fn legacy_prune_match_count(results: &LegacyResults, active_nodes: &ActiveNodes) -> usize {
    let active_nodes = active_nodes
        .iter()
        .map(|(entity, node_id)| (7, *entity, node_id.clone()))
        .collect::<BTreeSet<_>>();
    results
        .keys()
        .filter(|(world, entity, node_id)| {
            active_nodes.contains(&(*world, *entity, node_id.clone()))
        })
        .count()
}

fn grouped_prune_match_count(results: &GroupedResults, active_nodes: &ActiveNodes) -> usize {
    results
        .iter()
        .filter_map(|((world, entity), node_results)| {
            (*world == 7)
                .then(|| active_nodes.get(entity))
                .flatten()
                .and_then(|active_node| node_results.get(active_node))
        })
        .count()
}

fn benchmark_paired_samples(
    mut legacy: impl FnMut() -> u64,
    mut optimized: impl FnMut() -> u64,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    for sample_index in 0..BENCHMARK_SAMPLES {
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

fn print_performance_result(
    name: &str,
    legacy_samples: &[u128],
    optimized_samples: &[u128],
    dimensions: String,
    maximum_numerator: u128,
    maximum_denominator: u128,
) {
    let legacy_p50 = percentile(legacy_samples, 50);
    let legacy_p95 = percentile(legacy_samples, 95);
    let optimized_p50 = percentile(optimized_samples, 50);
    let optimized_p95 = percentile(optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(legacy_samples);
    let optimized_ns = benchmark_samples_csv(optimized_samples);
    println!(
        "PERF_RESULT {name} {dimensions} samples={BENCHMARK_SAMPLES} sample_pairs={BENCHMARK_SAMPLES} sample_order=alternating percentile_method=nearest_rank legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
    );
    assert!(
        optimized_p95 * maximum_denominator <= legacy_p95 * maximum_numerator,
        "optimized P95 {optimized_p95}ns must be no more than {maximum_numerator}/{maximum_denominator} of legacy P95 {legacy_p95}ns"
    );
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
