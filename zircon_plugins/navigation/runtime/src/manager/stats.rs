use zircon_runtime::core::framework::navigation::{
    NAV_MESH_AGENT_COMPONENT_TYPE, NAV_MESH_OBSTACLE_COMPONENT_TYPE,
    NAV_MESH_OFF_MESH_BRIDGE_COMPONENT_TYPE, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
    NavigationRuntimeStats,
};
use zircon_runtime::scene::World;

pub(super) fn count_obstacles(world: &World) -> usize {
    world
        .node_records()
        .into_iter()
        .filter(|node| {
            world
                .dynamic_component(node.id, NAV_MESH_OBSTACLE_COMPONENT_TYPE)
                .is_some()
        })
        .count()
}

pub(super) fn count_navigation_components(world: &World) -> NavigationRuntimeStats {
    let mut stats = NavigationRuntimeStats::default();
    // Use fresh projections so editor/runtime dynamic component writes are counted immediately.
    for node in world.node_records() {
        if world
            .dynamic_component(node.id, NAV_MESH_AGENT_COMPONENT_TYPE)
            .is_some()
        {
            stats.active_agents += 1;
        }
        if world
            .dynamic_component(node.id, NAV_MESH_OBSTACLE_COMPONENT_TYPE)
            .is_some()
        {
            stats.active_obstacles += 1;
        }
        if world
            .dynamic_component(node.id, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE)
            .is_some()
        {
            stats.active_off_mesh_links += 1;
        }
        if world
            .dynamic_component(node.id, NAV_MESH_OFF_MESH_BRIDGE_COMPONENT_TYPE)
            .is_some()
        {
            stats.active_off_mesh_bridges += 1;
        }
    }
    stats
}

#[cfg(test)]
mod tests {
    use std::{hint::black_box, time::Instant};

    use serde_json::json;
    use zircon_runtime::core::framework::navigation::{
        NAV_MESH_AGENT_COMPONENT_TYPE, NAV_MESH_OBSTACLE_COMPONENT_TYPE,
        NAV_MESH_OFF_MESH_BRIDGE_COMPONENT_TYPE, NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
        NavigationRuntimeStats,
    };
    use zircon_runtime::scene::{NodeKind, World};

    use super::count_navigation_components;

    const BENCHMARK_NODE_COUNT: usize = 4_096;
    const BENCHMARK_SAMPLE_COUNT: usize = 21;
    const COMPONENT_TYPES: [&str; 4] = [
        NAV_MESH_AGENT_COMPONENT_TYPE,
        NAV_MESH_OBSTACLE_COMPONENT_TYPE,
        NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
        NAV_MESH_OFF_MESH_BRIDGE_COMPONENT_TYPE,
    ];

    #[test]
    fn single_pass_navigation_stats_preserve_all_component_counts() {
        let world = benchmark_world(64);
        let legacy = legacy_count_navigation_components(&world);
        let optimized = count_navigation_components(&world);

        assert_eq!(stats_tuple(&optimized), stats_tuple(&legacy));
    }

    #[test]
    fn navigation_stats_use_one_world_projection_for_all_component_types() {
        let source = include_str!("stats.rs");
        let function = source
            .split("pub(super) fn count_navigation_components")
            .nth(1)
            .and_then(|body| body.split("#[cfg(test)]").next())
            .expect("navigation stats source");

        assert_eq!(function.matches(".node_records()").count(), 1);
        assert!(!function.contains("count_off_mesh_links"));
        assert!(!function.contains("count_off_mesh_bridges"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn single_pass_navigation_stats_release_benchmark_evidence() {
        let world = benchmark_world(BENCHMARK_NODE_COUNT);
        let legacy = legacy_count_navigation_components(&world);
        let optimized = count_navigation_components(&world);
        assert_eq!(stats_tuple(&optimized), stats_tuple(&legacy));

        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || legacy_count_navigation_components(&world),
            || count_navigation_components(&world),
        );
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_ns = benchmark_samples_csv(&optimized_samples);

        println!(
            "PERF_RESULT plugins14_single_pass_navigation_stats nodes={} samples={} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_world_projections=4 optimized_world_projections=1 legacy_projected_nodes={} optimized_projected_nodes={} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={legacy_ns} optimized_ns={optimized_ns}",
            BENCHMARK_NODE_COUNT,
            BENCHMARK_SAMPLE_COUNT,
            BENCHMARK_NODE_COUNT * COMPONENT_TYPES.len(),
            BENCHMARK_NODE_COUNT,
            legacy_p50,
            legacy_p95,
            optimized_p50,
            optimized_p95,
        );
        assert!(
            optimized_p95 * 5 <= legacy_p95 * 2,
            "single-pass stats P95 {optimized_p95}ns must be no more than 40% of legacy P95 {legacy_p95}ns"
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
            world
                .set_dynamic_component(
                    entity,
                    COMPONENT_TYPES[index % COMPONENT_TYPES.len()],
                    json!({}),
                )
                .unwrap();
        }
        world
    }

    fn legacy_count_navigation_components(world: &World) -> NavigationRuntimeStats {
        NavigationRuntimeStats {
            active_agents: count_component_instances(world, NAV_MESH_AGENT_COMPONENT_TYPE),
            active_obstacles: count_component_instances(world, NAV_MESH_OBSTACLE_COMPONENT_TYPE),
            active_off_mesh_links: count_component_instances(
                world,
                NAV_MESH_OFF_MESH_LINK_COMPONENT_TYPE,
            ),
            active_off_mesh_bridges: count_component_instances(
                world,
                NAV_MESH_OFF_MESH_BRIDGE_COMPONENT_TYPE,
            ),
            ..NavigationRuntimeStats::default()
        }
    }

    fn count_component_instances(world: &World, component_type: &str) -> usize {
        world
            .node_records()
            .into_iter()
            .filter(|node| world.dynamic_component(node.id, component_type).is_some())
            .count()
    }

    fn stats_tuple(stats: &NavigationRuntimeStats) -> (usize, usize, usize, usize) {
        (
            stats.active_agents,
            stats.active_obstacles,
            stats.active_off_mesh_links,
            stats.active_off_mesh_bridges,
        )
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
}
