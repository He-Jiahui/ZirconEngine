use zircon_runtime::core::framework::ai::{AiBehaviorAbortPolicy, AiBehaviorNodeParameterValue};

use crate::behavior_tree::BehaviorIntegrationTaskContext;

use super::{
    decorator_condition_passes, BehaviorNodeRuntimeState, BehaviorNodeSemantics,
    BehaviorNodeTickContext, BehaviorTreeExecutionContext, BehaviorTreeInstanceState,
    CompiledBehaviorTree, SUBTREE_TARGET_PARAMETER_KEY,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AbortRequest {
    SelfSubtree { node_index: u32 },
    LowerPriority { observer_index: u32 },
}

impl AbortRequest {
    const fn priority(self) -> u32 {
        match self {
            Self::SelfSubtree { node_index } => node_index,
            Self::LowerPriority { observer_index } => observer_index,
        }
    }
}

pub(super) fn process_observer_aborts(
    tree: &CompiledBehaviorTree,
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
) {
    let observer_pass = context.observer_pass;
    if context.changed_slots.is_empty()
        || !context
            .instance
            .mark_observers_processed(tree.id(), observer_pass)
    {
        return;
    }
    let mut observers = std::mem::take(&mut context.instance.observer_scratch);
    observers.clear();
    if let Some(set) = context.instance.observers.get(tree.id()) {
        set.append_matching(context.changed_slots, &mut observers);
    }
    let mut requests = std::mem::take(&mut context.instance.abort_request_scratch);
    requests.clear();
    requests.reserve(observers.len());
    for observer in observers.iter().copied() {
        let node = tree.node(observer.node_index as usize);
        let dense_value = context.dense_blackboard_value(tree.id(), observer.node_index);
        let condition_passes = decorator_condition_passes(
            node,
            context.blackboard,
            context.perception,
            dense_value.as_ref().map(Option::as_ref),
        );
        if matches!(
            observer.policy,
            AiBehaviorAbortPolicy::Self_ | AiBehaviorAbortPolicy::Both
        ) && !condition_passes
            && context
                .instance
                .node_mut(tree, observer.node_index)
                .is_active
        {
            requests.push(AbortRequest::SelfSubtree {
                node_index: observer.node_index,
            });
        }
        if matches!(
            observer.policy,
            AiBehaviorAbortPolicy::LowerPriority | AiBehaviorAbortPolicy::Both
        ) && condition_passes
        {
            requests.push(AbortRequest::LowerPriority {
                observer_index: observer.node_index,
            });
        }
    }
    context.instance.observer_scratch = observers;
    // Each compiled node owns one observer, so emitted priorities are unique.
    requests.sort_unstable_by_key(|request| request.priority());
    for request in requests.iter().copied() {
        match request {
            AbortRequest::SelfSubtree { node_index } => {
                abort_subtree(tree, node_index, context);
            }
            AbortRequest::LowerPriority { observer_index } => {
                abort_lower_priority_branch(tree, observer_index, context);
            }
        }
    }
    context.instance.abort_request_scratch = requests;
}

pub(super) fn abort_active_root(context: &mut BehaviorTreeExecutionContext<'_, '_>) {
    let Some(root_tree_id) = context.instance.root_tree.clone() else {
        return;
    };
    let Some(root_tree) = context
        .tree_descriptors
        .iter()
        .find(|tree| tree.id() == root_tree_id)
        .cloned()
    else {
        return;
    };
    abort_subtree(&root_tree, 0, context);
}

fn abort_lower_priority_branch(
    tree: &CompiledBehaviorTree,
    observer_index: u32,
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
) {
    let Some((selector_index, observer_branch)) = selector_ancestor(tree, observer_index) else {
        return;
    };
    let active_branch = context.instance.node_mut(tree, selector_index).active_child;
    let Some(active_branch) = active_branch else {
        return;
    };
    let selector = tree.node(selector_index as usize);
    let children = tree.child_indices(selector);
    let observer_priority = children.iter().position(|child| *child == observer_branch);
    let active_priority = children.iter().position(|child| *child == active_branch);
    if !matches!((observer_priority, active_priority), (Some(observer), Some(active)) if active > observer)
    {
        return;
    }
    abort_subtree(tree, active_branch, context);
    clear_node_control_state(context.instance.node_mut(tree, selector_index));
    clear_ancestor_control_state(tree, selector_index, context.instance);
}

pub(super) fn abort_subtree(
    tree: &CompiledBehaviorTree,
    root_index: u32,
    context: &mut BehaviorTreeExecutionContext<'_, '_>,
) {
    let range = tree.node(root_index as usize).subtree_range(root_index);
    for node_index in range {
        let node = tree.node(node_index as usize);
        let subtree_target = (node.semantics() == BehaviorNodeSemantics::RunSubtree)
            .then(|| subtree_target(node))
            .flatten();
        let (was_active, runtime) = {
            let state = context.instance.node_mut(tree, node_index);
            let was_active = state.is_active;
            state.is_active = false;
            state.elapsed_seconds = 0.0;
            state.loop_count = 0;
            state.selected_child = None;
            state.active_child = None;
            state.terminal_children.clear();
            (was_active, state.external_runtime.take())
        };
        if was_active {
            if let Some(target_tree) = subtree_target.and_then(|target| {
                context
                    .tree_descriptors
                    .iter()
                    .find(|candidate| candidate.id() == target)
                    .cloned()
            }) {
                abort_subtree(&target_tree, 0, context);
            }
        }
        if was_active {
            if matches!(
                node.semantics(),
                BehaviorNodeSemantics::MoveTo
                    | BehaviorNodeSemantics::PlayAnimation
                    | BehaviorNodeSemantics::ScriptTask
            ) {
                let abort_context = BehaviorIntegrationTaskContext {
                    node_id: node.id(),
                    parameters: node.parameters(),
                    entity: context.entity,
                    delta_seconds: context.delta_seconds,
                    started: false,
                };
                if let Some(host) = context.integration_host.as_deref_mut() {
                    host.abort(&abort_context);
                }
            }
            let Some(mut runtime) = runtime else {
                continue;
            };
            let abort_context = BehaviorNodeTickContext::new(
                node.parameters(),
                context.blackboard,
                context.perception,
                context.delta_seconds,
            );
            runtime.on_abort(&abort_context);
        }
    }
}

fn clear_ancestor_control_state(
    tree: &CompiledBehaviorTree,
    node_index: u32,
    instance: &mut BehaviorTreeInstanceState,
) {
    let mut child = node_index;
    while let Some(parent) = parent_of(tree, child) {
        clear_node_control_state(instance.node_mut(tree, parent));
        child = parent;
    }
}

fn clear_node_control_state(state: &mut BehaviorNodeRuntimeState) {
    state.is_active = false;
    state.elapsed_seconds = 0.0;
    state.loop_count = 0;
    state.selected_child = None;
    state.active_child = None;
    state.terminal_children.clear();
}

fn subtree_target(node: &super::CompiledBehaviorNode) -> Option<&str> {
    node.parameters()
        .iter()
        .find(|parameter| parameter.key == SUBTREE_TARGET_PARAMETER_KEY)
        .and_then(|parameter| match &parameter.value {
            AiBehaviorNodeParameterValue::String(target) => Some(target.as_str()),
            _ => None,
        })
}

fn selector_ancestor(tree: &CompiledBehaviorTree, node_index: u32) -> Option<(u32, u32)> {
    let mut branch = node_index;
    while let Some(parent) = parent_of(tree, branch) {
        if tree.node(parent as usize).semantics() == BehaviorNodeSemantics::Selector {
            return Some((parent, branch));
        }
        branch = parent;
    }
    None
}

fn parent_of(tree: &CompiledBehaviorTree, node_index: u32) -> Option<u32> {
    tree.parent_index(node_index)
}

#[cfg(test)]
mod parent_index_contract_tests {
    #[test]
    fn abort_parent_lookup_uses_the_compiled_parent_index() {
        let source = include_str!("abort.rs");
        let parent_of = source
            .split("fn parent_of(")
            .nth(1)
            .and_then(|body| body.split("\n}").next())
            .expect("parent_of body");

        assert!(parent_of.contains("tree.parent_index(node_index)"));
        assert!(!parent_of.contains("tree.nodes().iter()"));
        assert!(!parent_of.contains(".contains(&node_index)"));
    }
}

#[cfg(test)]
mod abort_request_performance_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::AbortRequest;

    const BENCHMARK_REQUEST_COUNT: usize = 8_192;
    const BENCHMARK_SAMPLE_COUNT: usize = 21;
    const REUSE_BENCHMARK_REQUEST_COUNT: usize = 32;
    const REUSE_BENCHMARK_ITERATIONS: usize = 4_096;

    #[test]
    fn unstable_abort_request_sort_preserves_unique_priority_order() {
        let requests = benchmark_requests(64);
        assert_eq!(legacy_sort(&requests), optimized_sort(&requests));
        assert!(optimized_sort(&requests)
            .windows(2)
            .all(|pair| pair[0].priority() < pair[1].priority()));
    }

    #[test]
    fn observer_abort_processing_preallocates_and_uses_in_place_sort() {
        let source = include_str!("abort.rs");
        let processing = source
            .split("pub(super) fn process_observer_aborts(")
            .nth(1)
            .and_then(|body| body.split("pub(super) fn abort_active_root").next())
            .expect("observer abort processing body");

        assert!(processing.contains("std::mem::take(&mut context.instance.abort_request_scratch)"));
        assert!(processing.contains("requests.reserve(observers.len())"));
        assert!(processing.contains("requests.sort_unstable_by_key"));
        assert!(processing.contains("context.instance.abort_request_scratch = requests"));
        assert!(!processing.contains("requests.sort_by_key"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn in_place_abort_request_sort_release_benchmark_evidence() {
        let requests = benchmark_requests(BENCHMARK_REQUEST_COUNT);
        assert_eq!(legacy_sort(&requests), optimized_sort(&requests));

        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || black_box(legacy_sort(black_box(&requests))).len(),
            || black_box(optimized_sort(black_box(&requests))).len(),
        );
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_ns = benchmark_samples_csv(&optimized_samples);

        println!(
            "PERF_RESULT plugins15_in_place_abort_request_sort requests={BENCHMARK_REQUEST_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank priorities=unique legacy_stable_sort=1 optimized_stable_sort=0 optimized_in_place_sort=1 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
        );
        assert!(
            optimized_p95 * 5 <= legacy_p95 * 4,
            "optimized P95 {optimized_p95}ns must be no more than 80% of legacy P95 {legacy_p95}ns"
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn reusable_abort_request_scratch_release_benchmark_evidence() {
        let requests = benchmark_requests(REUSE_BENCHMARK_REQUEST_COUNT);
        let mut scratch = Vec::new();
        optimized_sort_into(&requests, &mut scratch);
        assert_eq!(scratch, legacy_sort(&requests));

        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || {
                let mut sorted = 0_usize;
                for _ in 0..REUSE_BENCHMARK_ITERATIONS {
                    sorted += black_box(legacy_sort(black_box(&requests))).len();
                }
                sorted
            },
            || {
                let mut sorted = 0_usize;
                for _ in 0..REUSE_BENCHMARK_ITERATIONS {
                    optimized_sort_into(black_box(&requests), &mut scratch);
                    sorted += black_box(scratch.len());
                }
                sorted
            },
        );
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_ns = benchmark_samples_csv(&optimized_samples);

        println!(
            "PERF_RESULT plugins15_reusable_abort_request_scratch requests={REUSE_BENCHMARK_REQUEST_COUNT} iterations_per_sample={REUSE_BENCHMARK_ITERATIONS} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank priorities=unique legacy_request_vec_allocations_per_sample={REUSE_BENCHMARK_ITERATIONS} optimized_request_vec_allocations_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
        );
        assert!(
            optimized_p95 * 5 <= legacy_p95 * 4,
            "optimized P95 {optimized_p95}ns must be no more than 80% of legacy P95 {legacy_p95}ns"
        );
    }

    fn benchmark_requests(request_count: usize) -> Vec<AbortRequest> {
        (0..request_count as u32)
            .rev()
            .map(|priority| {
                if priority % 2 == 0 {
                    AbortRequest::SelfSubtree {
                        node_index: priority,
                    }
                } else {
                    AbortRequest::LowerPriority {
                        observer_index: priority,
                    }
                }
            })
            .collect()
    }

    fn legacy_sort(requests: &[AbortRequest]) -> Vec<AbortRequest> {
        let mut requests = requests.to_vec();
        requests.sort_by_key(|request| request.priority());
        requests
    }

    fn optimized_sort(requests: &[AbortRequest]) -> Vec<AbortRequest> {
        let mut requests = requests.to_vec();
        requests.sort_unstable_by_key(|request| request.priority());
        requests
    }

    fn optimized_sort_into(requests: &[AbortRequest], scratch: &mut Vec<AbortRequest>) {
        scratch.clear();
        scratch.extend_from_slice(requests);
        scratch.sort_unstable_by_key(|request| request.priority());
    }

    fn benchmark_paired_samples(
        mut legacy: impl FnMut() -> usize,
        mut optimized: impl FnMut() -> usize,
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

    fn benchmark_sample(operation: &mut impl FnMut() -> usize) -> u128 {
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
}
