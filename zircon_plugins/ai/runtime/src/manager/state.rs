use std::collections::HashMap;
use std::sync::Arc;

use zircon_runtime::core::framework::ai::{
    AiAgentTickReport, AiBehaviorTreeDescriptor, AiBehaviorTreeId, AiBlackboardEntry,
    AiBlackboardSchemaDescriptor, AiBlackboardSchemaId, AiPerceptionSnapshot,
};
use zircon_runtime::core::framework::scene::{EntityId, WorldHandle};

use crate::behavior_tree::{BehaviorTreeInstanceState, CompiledBehaviorTree};
use crate::blackboard::{BlackboardLayout, BlackboardStore};

#[derive(Debug, Default)]
pub(super) struct AiRuntimeState {
    pub(super) next_behavior_tree_id: u64,
    pub(super) next_blackboard_schema_id: u64,
    pub(super) behavior_trees: Vec<RegisteredBehaviorTree>,
    pub(super) compiled_behavior_tree_generation: Arc<[CompiledBehaviorTree]>,
    pub(super) blackboard_schemas: Vec<RegisteredBlackboardSchema>,
    pub(super) blackboards: HashMap<(WorldHandle, EntityId), AgentBlackboard>,
    pub(super) perceptions: HashMap<(WorldHandle, EntityId), AiPerceptionSnapshot>,
    pub(super) active_behavior_trees: HashMap<(WorldHandle, EntityId), ActiveBehaviorAgent>,
    pub(super) behavior_tree_instances: HashMap<(WorldHandle, EntityId), BehaviorTreeInstanceState>,
    pub(super) last_reports: HashMap<(WorldHandle, EntityId), AiAgentTickReport>,
}

impl AiRuntimeState {
    pub(super) fn rebuild_compiled_behavior_tree_generation(&mut self) {
        self.compiled_behavior_tree_generation = self
            .behavior_trees
            .iter()
            .map(|entry| entry.compiled.clone())
            .collect::<Vec<_>>()
            .into();
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ActiveBehaviorAgent {
    pub(super) behavior_tree: AiBehaviorTreeId,
    pub(super) blackboard_schema: Option<AiBlackboardSchemaId>,
    pub(super) pending_delta_seconds: f32,
}

#[derive(Clone, Debug)]
pub(super) struct RegisteredBehaviorTree {
    pub(super) id: AiBehaviorTreeId,
    pub(super) descriptor: AiBehaviorTreeDescriptor,
    pub(super) compiled: CompiledBehaviorTree,
}

#[derive(Clone, Debug)]
pub(super) struct RegisteredBlackboardSchema {
    pub(super) id: AiBlackboardSchemaId,
    pub(super) descriptor: AiBlackboardSchemaDescriptor,
    pub(super) layout: Arc<BlackboardLayout>,
}

#[derive(Clone, Debug)]
pub(super) enum AgentBlackboard {
    Dynamic(Vec<AiBlackboardEntry>),
    Dense(BlackboardStore),
}

impl AgentBlackboard {
    pub(super) fn entries(&self) -> Vec<AiBlackboardEntry> {
        match self {
            Self::Dynamic(entries) => entries.clone(),
            Self::Dense(store) => store.entries(),
        }
    }

    pub(super) fn entries_ref(&self) -> &[AiBlackboardEntry] {
        match self {
            Self::Dynamic(entries) => entries,
            Self::Dense(store) => store.entries_ref(),
        }
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::sync::Arc;
    use std::time::Instant;

    use zircon_runtime::core::framework::ai::{
        AiBehaviorNodeDescriptor, AiBehaviorNodeKind, AiBehaviorTreeDescriptor, AiBehaviorTreeId,
    };

    use super::{AiRuntimeState, RegisteredBehaviorTree};
    use crate::behavior_tree::compile_behavior_tree;

    const TREE_COUNT: usize = 256;
    const NODES_PER_TREE: usize = 32;
    const BENCHMARK_SAMPLES: usize = 21;
    const BENCHMARK_ITERATIONS: usize = 32;

    #[test]
    fn compiled_tree_generation_rebuilds_in_registry_order() {
        let mut state = AiRuntimeState::default();
        state.behavior_trees = (0..3).map(registered_tree).collect();

        state.rebuild_compiled_behavior_tree_generation();

        assert_eq!(
            state
                .compiled_behavior_tree_generation
                .iter()
                .map(|tree| tree.id())
                .collect::<Vec<_>>(),
            ["tree-0", "tree-1", "tree-2"]
        );
    }

    #[test]
    fn steady_tick_uses_compiled_tree_generation_without_deep_catalog_clone() {
        let source = include_str!("tick.rs");
        assert!(source.contains("Arc::clone(&state.compiled_behavior_tree_generation)"));
        assert!(!source.contains(".map(|entry| entry.compiled.clone())"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn immutable_compiled_tree_generation_release_benchmark_evidence() {
        let mut state = AiRuntimeState::default();
        state.behavior_trees = (0..TREE_COUNT).map(registered_tree).collect();
        state.rebuild_compiled_behavior_tree_generation();

        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || {
                for _ in 0..BENCHMARK_ITERATIONS {
                    black_box(
                        state
                            .behavior_trees
                            .iter()
                            .map(|entry| entry.compiled.clone())
                            .collect::<Vec<_>>(),
                    );
                }
            },
            || {
                for _ in 0..BENCHMARK_ITERATIONS {
                    black_box(Arc::clone(&state.compiled_behavior_tree_generation));
                }
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
            "PERF_RESULT plugins15_immutable_compiled_tree_generation trees={} nodes_per_tree={} iterations={} sample_pairs={BENCHMARK_SAMPLES} sample_order=alternating percentile_method=nearest_rank legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} p95_ratio={ratio:.6} legacy_ns={legacy_ns} optimized_ns={optimized_ns}",
            TREE_COUNT,
            NODES_PER_TREE,
            BENCHMARK_ITERATIONS,
            legacy_p50,
            legacy_p95,
            optimized_p50,
            optimized_p95,
        );
        assert!(
            optimized_p95.saturating_mul(10) <= legacy_p95,
            "immutable generation P95 must be at most 10% of deep-clone P95"
        );
    }

    fn benchmark_samples_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn registered_tree(index: usize) -> RegisteredBehaviorTree {
        let tree_name = format!("tree-{index}");
        let mut descriptor =
            AiBehaviorTreeDescriptor::new(tree_name.as_str(), tree_name.as_str(), "node-0");
        for node_index in 0..NODES_PER_TREE {
            let node_name = format!("node-{node_index}");
            let mut node = AiBehaviorNodeDescriptor::new(
                node_name.as_str(),
                if node_index + 1 == NODES_PER_TREE {
                    AiBehaviorNodeKind::Task
                } else {
                    AiBehaviorNodeKind::Sequence
                },
                node_name.as_str(),
            );
            if node_index + 1 < NODES_PER_TREE {
                node = node.with_child(format!("node-{}", node_index + 1));
            }
            descriptor = descriptor.with_node(node);
        }
        let compiled = compile_behavior_tree(&descriptor).expect("benchmark tree compiles");
        RegisteredBehaviorTree {
            id: AiBehaviorTreeId::new(index as u64 + 1),
            descriptor,
            compiled,
        }
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
