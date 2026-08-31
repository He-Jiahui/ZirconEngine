use std::collections::HashSet;

use zircon_runtime::core::framework::ai::{AiAgentRuntimeSnapshot, AiRuntimeSnapshot};
use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};

use super::state::AiRuntimeState;
use super::DefaultAiManager;

pub(super) fn runtime_snapshot(manager: &DefaultAiManager) -> AiRuntimeSnapshot {
    let state = manager.lock_state();
    build_runtime_snapshot(&state)
}

fn build_runtime_snapshot(state: &AiRuntimeState) -> AiRuntimeSnapshot {
    let agent_keys = state
        .blackboards
        .keys()
        .chain(state.perceptions.keys())
        .chain(state.active_behavior_trees.keys())
        .chain(state.last_reports.keys())
        .copied()
        .collect::<HashSet<_>>();
    let agents = agent_keys
        .into_iter()
        .filter_map(|key| build_agent_runtime_snapshot(state, key))
        .collect();

    AiRuntimeSnapshot {
        behavior_trees: state
            .behavior_trees
            .iter()
            .map(|entry| entry.descriptor.clone())
            .collect(),
        agents,
    }
}

pub(super) fn runtime_snapshots_for_agents(
    manager: &DefaultAiManager,
    world: WorldHandle,
    entities: impl IntoIterator<Item = EntityId>,
) -> Vec<AiAgentRuntimeSnapshot> {
    let state = manager.lock_state();
    build_agent_runtime_snapshots(&state, world, entities)
}

fn build_agent_runtime_snapshots(
    state: &AiRuntimeState,
    world: WorldHandle,
    entities: impl IntoIterator<Item = EntityId>,
) -> Vec<AiAgentRuntimeSnapshot> {
    entities
        .into_iter()
        .filter_map(|entity| build_agent_runtime_snapshot(state, (world, entity)))
        .collect()
}

fn build_agent_runtime_snapshot(
    state: &AiRuntimeState,
    (world, entity): (WorldHandle, EntityId),
) -> Option<AiAgentRuntimeSnapshot> {
    let key = (world, entity);
    if !state.blackboards.contains_key(&key)
        && !state.perceptions.contains_key(&key)
        && !state.active_behavior_trees.contains_key(&key)
        && !state.last_reports.contains_key(&key)
    {
        return None;
    }
    Some(AiAgentRuntimeSnapshot {
        world,
        entity,
        behavior_tree: state.active_behavior_trees.get(&key).and_then(|active| {
            state
                .behavior_trees
                .iter()
                .find(|tree| tree.id == active.behavior_tree)
                .map(|tree| tree.descriptor.id.clone())
        }),
        blackboard: state
            .blackboards
            .get(&key)
            .map(|blackboard| blackboard.entries())
            .unwrap_or_default(),
        perception: state.perceptions.get(&key).cloned(),
    })
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime::core::framework::ai::{
        AiAgentTickReport, AiBlackboardEntry, AiBlackboardValue,
    };
    use zircon_runtime::core::framework::scene::WorldHandle;

    use super::{build_agent_runtime_snapshots, build_runtime_snapshot};
    use crate::manager::state::{AgentBlackboard, AiRuntimeState};

    const AGENT_COUNT: u64 = 8_192;
    const ACTIVE_AGENT_COUNT: u64 = 256;
    const BENCHMARK_SAMPLES: usize = 21;

    #[test]
    fn targeted_projection_reads_only_requested_world_agents() {
        let first_world = WorldHandle::new(1);
        let second_world = WorldHandle::new(2);
        let mut state = AiRuntimeState::default();
        insert_agent(&mut state, first_world, 1);
        insert_agent(&mut state, first_world, 2);
        insert_agent(&mut state, second_world, 2);

        let snapshots = build_agent_runtime_snapshots(&state, first_world, [2]);

        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].world, first_world);
        assert_eq!(snapshots[0].entity, 2);
        assert_eq!(snapshots[0].blackboard.len(), 1);
    }

    #[test]
    fn extracted_agent_projection_retains_last_report_only_agents() {
        let world = WorldHandle::new(1);
        let mut state = AiRuntimeState::default();
        state
            .last_reports
            .insert((world, 7), AiAgentTickReport::idle(world, 7));

        let full = build_runtime_snapshot(&state);
        let targeted = build_agent_runtime_snapshots(&state, world, [7]);

        assert_eq!(full.agents.len(), 1);
        assert_eq!(full.agents[0].entity, 7);
        assert_eq!(targeted.len(), 1);
        assert_eq!(targeted[0].entity, 7);
    }

    #[test]
    fn behavior_tick_uses_targeted_debug_projection() {
        let source = include_str!("../plugin/registration.rs");
        assert!(source.contains("runtime_snapshots_for_agents"));
        assert!(!source.contains(".runtime_snapshot()"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn targeted_debug_snapshot_release_benchmark_evidence() {
        let target_world = WorldHandle::new(1);
        let other_world = WorldHandle::new(2);
        let mut state = AiRuntimeState::default();
        for entity in 0..AGENT_COUNT {
            let world = if entity % 2 == 0 {
                target_world
            } else {
                other_world
            };
            insert_agent(&mut state, world, entity);
        }
        let active_agents = (0..ACTIVE_AGENT_COUNT)
            .map(|index| index * 2)
            .collect::<Vec<_>>();

        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || black_box(build_runtime_snapshot(black_box(&state))),
            || {
                black_box(build_agent_runtime_snapshots(
                    black_box(&state),
                    target_world,
                    black_box(active_agents.iter().copied()),
                ))
            },
        );
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let ratio = optimized_p95 as f64 / legacy_p95.max(1) as f64;
        let legacy_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_ns = benchmark_samples_csv(&optimized_samples);
        println!(
            "PERF_RESULT plugins15_targeted_debug_snapshot total_agents={} active_agents={} sample_pairs={BENCHMARK_SAMPLES} sample_order=alternating percentile_method=nearest_rank legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} p95_ratio={ratio:.6} legacy_ns={legacy_ns} optimized_ns={optimized_ns}",
            AGENT_COUNT, ACTIVE_AGENT_COUNT, legacy_p50, legacy_p95, optimized_p50, optimized_p95,
        );
        assert!(
            optimized_p95.saturating_mul(4) <= legacy_p95,
            "targeted debug snapshot P95 must be at most 25% of full snapshot P95"
        );
    }

    fn benchmark_samples_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn insert_agent(state: &mut AiRuntimeState, world: WorldHandle, entity: u64) {
        state.blackboards.insert(
            (world, entity),
            AgentBlackboard::Dynamic(vec![AiBlackboardEntry::new(
                "entity",
                AiBlackboardValue::Entity(entity),
            )]),
        );
    }

    fn benchmark_paired_samples<L, O>(
        mut legacy: impl FnMut() -> L,
        mut optimized: impl FnMut() -> O,
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
