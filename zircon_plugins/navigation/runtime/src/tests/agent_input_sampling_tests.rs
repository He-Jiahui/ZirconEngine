use std::{hint::black_box, time::Instant};

use serde_json::json;
use zircon_runtime::core::framework::navigation::{
    NavMeshAgentDescriptor, NAV_MESH_AGENT_COMPONENT_TYPE, NAV_MESH_OBSTACLE_COMPONENT_TYPE,
};
use zircon_runtime::scene::{NodeKind, World};

use super::collect_agent_tick_inputs;
use crate::component_json::parse_component;

const BENCHMARK_NODE_COUNT: usize = 4_096;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn single_pass_agent_tick_inputs_preserve_agents_and_obstacle_detection() {
    let world = benchmark_world(64);
    let legacy = legacy_collect_agent_tick_inputs(&world);
    let optimized = collect_agent_tick_inputs(&world);

    assert_eq!(optimized, legacy);
}

#[test]
fn agent_tick_inputs_use_one_world_projection() {
    let source = include_str!("../agent.rs");
    let tick = source
        .split("pub(super) fn tick_world_agents")
        .nth(1)
        .and_then(|body| body.split("fn ensure_crowd").next())
        .expect("agent tick source");
    let collector = source
        .split("fn collect_agent_tick_inputs")
        .nth(1)
        .and_then(|body| body.split("#[cfg(test)]").next())
        .expect("agent tick input collector source");

    assert!(tick.contains("collect_agent_tick_inputs(world)"));
    assert!(!tick.contains("collect_runtime_obstacles(world)"));
    assert_eq!(collector.matches(".node_records()").count(), 1);
}

#[test]
#[ignore = "release-only performance evidence"]
fn single_pass_agent_tick_inputs_release_benchmark_evidence() {
    let world = benchmark_world(BENCHMARK_NODE_COUNT);
    assert_eq!(
        collect_agent_tick_inputs(&world),
        legacy_collect_agent_tick_inputs(&world)
    );

    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || legacy_collect_agent_tick_inputs(&world),
        || collect_agent_tick_inputs(&world),
    );
    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let legacy_ns = benchmark_samples_csv(&legacy_samples);
    let optimized_ns = benchmark_samples_csv(&optimized_samples);

    println!(
        "PERF_RESULT plugins14_single_pass_agent_tick_inputs nodes={} samples={} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_world_projections=2 optimized_world_projections=1 legacy_projected_nodes={} optimized_projected_nodes={} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={legacy_ns} optimized_ns={optimized_ns}",
        BENCHMARK_NODE_COUNT,
        BENCHMARK_SAMPLE_COUNT,
        BENCHMARK_NODE_COUNT * 2,
        BENCHMARK_NODE_COUNT,
        legacy_p50,
        legacy_p95,
        optimized_p50,
        optimized_p95,
    );
    assert!(
        optimized_p95 * 10 <= legacy_p95 * 7,
        "single-pass input P95 {optimized_p95}ns must be no more than 70% of legacy P95 {legacy_p95}ns"
    );
}

fn benchmark_samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn benchmark_world(node_count: usize) -> World {
    let mut world = World::empty();
    for index in 0..node_count {
        let entity = world.spawn_node(NodeKind::Empty);
        if index % 2 == 0 {
            world
                .set_dynamic_component(entity, NAV_MESH_AGENT_COMPONENT_TYPE, json!({}))
                .unwrap();
        }
        if index % 3 == 0 {
            world
                .set_dynamic_component(entity, NAV_MESH_OBSTACLE_COMPONENT_TYPE, json!({}))
                .unwrap();
        }
    }
    world
}

fn legacy_collect_agent_tick_inputs(world: &World) -> (Vec<(u64, NavMeshAgentDescriptor)>, bool) {
    let agents = world
        .node_records()
        .into_iter()
        .filter_map(|node| {
            let value = world.dynamic_component(node.id, NAV_MESH_AGENT_COMPONENT_TYPE)?;
            Some((node.id, parse_component::<NavMeshAgentDescriptor>(value)))
        })
        .collect();
    let has_obstacles = world.node_records().into_iter().any(|node| {
        world
            .dynamic_component(node.id, NAV_MESH_OBSTACLE_COMPONENT_TYPE)
            .is_some()
    });
    (agents, has_obstacles)
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
    assert!(!ordered.is_empty());
    assert!((1..=100).contains(&percentile));
    let index = (ordered.len() * percentile).div_ceil(100) - 1;
    ordered[index]
}
