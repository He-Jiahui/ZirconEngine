use zircon_runtime::core::framework::ai::{
    AiBehaviorAbortPolicy, AiBehaviorNodeParameterValue, AiManagerError,
};

use crate::behavior_tree::CompiledBehaviorTree;

use super::{BlackboardLayout, BlackboardSlot};

const BLACKBOARD_KEY_PARAMETER: &str = "blackboard_key";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BlackboardObserver {
    pub(crate) node_index: u32,
    pub(crate) policy: AiBehaviorAbortPolicy,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct BlackboardObserverSet {
    schema_id: String,
    observers_by_slot: Box<[Box<[BlackboardObserver]>]>,
    slots_by_node: Box<[Option<BlackboardSlot>]>,
}

impl BlackboardObserverSet {
    pub(crate) fn resolve(
        tree: &CompiledBehaviorTree,
        layout: &BlackboardLayout,
    ) -> Result<Self, AiManagerError> {
        let mut observers_by_slot = std::iter::repeat_with(Vec::new)
            .take(layout.key_count())
            .collect::<Vec<_>>();
        let mut slots_by_node = vec![None; tree.nodes().len()];
        for (node_index, node) in tree.nodes().iter().enumerate() {
            let key = node.parameters().iter().find_map(|parameter| {
                (parameter.key == BLACKBOARD_KEY_PARAMETER)
                    .then_some(&parameter.value)
                    .and_then(AiBehaviorNodeParameterValue::as_string)
            });
            if node.abort_policy() != AiBehaviorAbortPolicy::None && key.is_none() {
                return Err(AiManagerError::BehaviorObserverMissingBlackboardKey {
                    tree_id: tree.id().to_string(),
                    node_id: node.id().to_string(),
                });
            }
            let Some(key) = key else {
                continue;
            };
            let slot = layout.resolve(key);
            if node.abort_policy() != AiBehaviorAbortPolicy::None && slot.is_none() {
                return Err(AiManagerError::BehaviorObserverUnknownBlackboardKey {
                    tree_id: tree.id().to_string(),
                    node_id: node.id().to_string(),
                    schema_id: layout.schema_id().to_string(),
                    key: key.to_string(),
                });
            }
            let Some(slot) = slot else {
                continue;
            };
            slots_by_node[node_index] = Some(slot);
            if node.abort_policy() == AiBehaviorAbortPolicy::None {
                continue;
            }
            observers_by_slot[slot.generation_index() as usize].push(BlackboardObserver {
                node_index: node_index as u32,
                policy: node.abort_policy(),
            });
        }
        Ok(Self {
            schema_id: layout.schema_id().to_string(),
            observers_by_slot: observers_by_slot
                .into_iter()
                .map(Vec::into_boxed_slice)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            slots_by_node: slots_by_node.into_boxed_slice(),
        })
    }

    pub(crate) fn schema_id(&self) -> &str {
        &self.schema_id
    }

    pub(crate) fn slot_for_node(&self, node_index: u32) -> Option<BlackboardSlot> {
        self.slots_by_node
            .get(node_index as usize)
            .copied()
            .flatten()
    }

    pub(crate) fn matching(&self, changed_slots: &[BlackboardSlot]) -> Vec<BlackboardObserver> {
        let observer_count = changed_slots
            .iter()
            .filter_map(|slot| self.observers_by_slot.get(slot.generation_index() as usize))
            .map(|observers| observers.len())
            .sum();
        let mut matching = Vec::with_capacity(observer_count);
        self.append_matching(changed_slots, &mut matching);
        matching
    }

    pub(crate) fn append_matching(
        &self,
        changed_slots: &[BlackboardSlot],
        matching: &mut Vec<BlackboardObserver>,
    ) {
        for slot in changed_slots {
            if let Some(observers) = self.observers_by_slot.get(slot.generation_index() as usize) {
                matching.extend_from_slice(observers);
            }
        }
    }
}

#[cfg(test)]
mod matching_performance_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime::core::framework::ai::{
        AiBehaviorAbortPolicy, AiBlackboardSchemaDescriptor,
    };

    use super::{BlackboardLayout, BlackboardObserver, BlackboardObserverSet, BlackboardSlot};

    const BENCHMARK_SLOT_COUNT: usize = 2_048;
    const BENCHMARK_OBSERVERS_PER_SLOT: usize = 64;
    const BENCHMARK_SAMPLE_COUNT: usize = 21;
    const REUSE_BENCHMARK_SLOT_COUNT: usize = 32;
    const REUSE_BENCHMARK_ITERATIONS: usize = 4_096;
    const SINGLE_PASS_BENCHMARK_SLOT_COUNT: usize = 2_048;
    const SINGLE_PASS_BENCHMARK_ITERATIONS: usize = 512;

    #[test]
    fn matching_preserves_changed_slot_and_registration_order() {
        let (observers, slots) = observer_fixture(3, 2);

        assert_eq!(
            observers
                .matching(&[slots[2], slots[0]])
                .into_iter()
                .map(|observer| observer.node_index)
                .collect::<Vec<_>>(),
            [4, 5, 0, 1]
        );
    }

    #[test]
    fn matching_preallocates_the_exact_observer_count() {
        let source = include_str!("observer.rs");
        let matching = source
            .split("pub(crate) fn matching(")
            .nth(1)
            .and_then(|body| body.split("\n    }").next())
            .expect("matching body");

        assert!(matching.contains("observer_count"));
        assert!(matching.contains("Vec::with_capacity(observer_count)"));
        assert!(!matching.contains(".flat_map("));
    }

    #[test]
    fn append_matching_reuses_capacity_and_preserves_order() {
        let (observers, slots) = observer_fixture(3, 2);
        let mut matching = Vec::new();

        observers.append_matching(&[slots[2], slots[0]], &mut matching);
        assert_eq!(
            matching
                .iter()
                .map(|observer| observer.node_index)
                .collect::<Vec<_>>(),
            [4, 5, 0, 1]
        );
        let capacity = matching.capacity();
        let allocation = matching.as_ptr();

        matching.clear();
        observers.append_matching(&[slots[1], slots[0]], &mut matching);
        assert_eq!(matching.capacity(), capacity);
        assert_eq!(matching.as_ptr(), allocation);
        assert_eq!(
            matching
                .iter()
                .map(|observer| observer.node_index)
                .collect::<Vec<_>>(),
            [2, 3, 0, 1]
        );
    }

    #[test]
    fn behavior_executor_reuses_observer_matching_scratch() {
        let executor = include_str!("../behavior_tree/executor.rs");
        let abort = include_str!("../behavior_tree/executor/abort.rs");

        assert!(executor.contains("observer_scratch: Vec<BlackboardObserver>"));
        assert!(abort.contains("std::mem::take(&mut context.instance.observer_scratch)"));
        assert!(abort.contains("set.append_matching(context.changed_slots, &mut observers)"));
        assert!(abort.contains("context.instance.observer_scratch = observers"));
    }

    #[test]
    fn append_matching_uses_one_changed_slot_pass_after_scratch_warmup() {
        let source = include_str!("observer.rs");
        let append = source
            .split("pub(crate) fn append_matching(")
            .nth(1)
            .and_then(|body| body.split("\n    }").next())
            .expect("append_matching body");

        assert!(!append.contains("observer_count"));
        assert!(!append.contains("matching.reserve("));
        assert!(append.contains("for slot in changed_slots"));
        assert!(append.contains("matching.extend_from_slice(observers)"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn preallocated_blackboard_observer_matching_release_benchmark_evidence() {
        let (observers, slots) =
            observer_fixture(BENCHMARK_SLOT_COUNT, BENCHMARK_OBSERVERS_PER_SLOT);
        let expected_observers = BENCHMARK_SLOT_COUNT * BENCHMARK_OBSERVERS_PER_SLOT;
        assert_eq!(observers.matching(&slots).len(), expected_observers);

        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || black_box(legacy_matching(black_box(&observers), black_box(&slots))).len(),
            || black_box(observers.matching(black_box(&slots))).len(),
        );
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_ns = benchmark_samples_csv(&optimized_samples);

        println!(
            "PERF_RESULT plugins15_preallocated_blackboard_observer_matching changed_slots={BENCHMARK_SLOT_COUNT} observers_per_slot={BENCHMARK_OBSERVERS_PER_SLOT} output_observers={expected_observers} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_capacity_precomputed=0 optimized_capacity_precomputed=1 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
        );
        assert!(
            optimized_p95 * 5 <= legacy_p95 * 4,
            "optimized P95 {optimized_p95}ns must be no more than 80% of legacy P95 {legacy_p95}ns"
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn reusable_blackboard_observer_matching_release_benchmark_evidence() {
        let (observers, slots) = observer_fixture(REUSE_BENCHMARK_SLOT_COUNT, 1);
        let mut scratch = Vec::new();
        observers.append_matching(&slots, &mut scratch);
        assert_eq!(scratch.len(), REUSE_BENCHMARK_SLOT_COUNT);
        scratch.clear();

        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || {
                let mut matched = 0_u64;
                for _ in 0..REUSE_BENCHMARK_ITERATIONS {
                    matched += black_box(observers.matching(black_box(&slots))).len() as u64;
                }
                matched
            },
            || {
                let mut matched = 0_u64;
                for _ in 0..REUSE_BENCHMARK_ITERATIONS {
                    scratch.clear();
                    observers.append_matching(black_box(&slots), &mut scratch);
                    matched += black_box(scratch.len()) as u64;
                }
                matched
            },
        );
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_ns = benchmark_samples_csv(&optimized_samples);

        println!(
            "PERF_RESULT plugins15_reusable_blackboard_observer_matching slots={REUSE_BENCHMARK_SLOT_COUNT} observers_per_slot=1 iterations_per_sample={REUSE_BENCHMARK_ITERATIONS} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_result_vec_allocations_per_sample={REUSE_BENCHMARK_ITERATIONS} optimized_result_vec_allocations_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
        );
        assert!(
            optimized_p95 * 5 <= legacy_p95 * 4,
            "optimized P95 {optimized_p95}ns must be no more than 80% of legacy P95 {legacy_p95}ns"
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn single_pass_reusable_observer_matching_release_benchmark_evidence() {
        let (observers, slots) = observer_fixture(SINGLE_PASS_BENCHMARK_SLOT_COUNT, 1);
        let mut legacy_scratch = Vec::new();
        let mut optimized_scratch = Vec::new();
        legacy_append_matching(&observers, &slots, &mut legacy_scratch);
        observers.append_matching(&slots, &mut optimized_scratch);
        assert_eq!(legacy_scratch, optimized_scratch);

        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || {
                let mut matched = 0_usize;
                for _ in 0..SINGLE_PASS_BENCHMARK_ITERATIONS {
                    legacy_scratch.clear();
                    legacy_append_matching(
                        black_box(&observers),
                        black_box(&slots),
                        &mut legacy_scratch,
                    );
                    matched += black_box(legacy_scratch.len());
                }
                matched
            },
            || {
                let mut matched = 0_usize;
                for _ in 0..SINGLE_PASS_BENCHMARK_ITERATIONS {
                    optimized_scratch.clear();
                    observers.append_matching(black_box(&slots), &mut optimized_scratch);
                    matched += black_box(optimized_scratch.len());
                }
                matched
            },
        );
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_ns = benchmark_samples_csv(&optimized_samples);

        println!(
            "PERF_RESULT plugins15_single_pass_reusable_observer_matching changed_slots={SINGLE_PASS_BENCHMARK_SLOT_COUNT} observers_per_slot=1 iterations_per_sample={SINGLE_PASS_BENCHMARK_ITERATIONS} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank scratch_warmed=1 legacy_changed_slot_passes_per_iteration=2 optimized_changed_slot_passes_per_iteration=1 legacy_result_vec_allocations_per_sample=0 optimized_result_vec_allocations_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
        );
        assert!(
            optimized_p95 * 4 <= legacy_p95 * 3,
            "optimized P95 {optimized_p95}ns must be no more than 75% of legacy P95 {legacy_p95}ns"
        );
    }

    fn observer_fixture(
        slot_count: usize,
        observers_per_slot: usize,
    ) -> (BlackboardObserverSet, Vec<BlackboardSlot>) {
        let mut descriptor = AiBlackboardSchemaDescriptor::new("benchmark", "Benchmark");
        for index in 0..slot_count {
            descriptor = descriptor.with_key(format!("key_{index:04}"), "integer", false);
        }
        let layout = BlackboardLayout::from_schema(&descriptor).expect("valid layout");
        let slots = descriptor
            .keys
            .iter()
            .map(|key| layout.resolve(&key.key).expect("compiled slot"))
            .collect::<Vec<_>>();
        let mut next_node_index = 0_u32;
        let observers_by_slot = (0..slot_count)
            .map(|_| {
                (0..observers_per_slot)
                    .map(|_| {
                        let observer = BlackboardObserver {
                            node_index: next_node_index,
                            policy: AiBehaviorAbortPolicy::Both,
                        };
                        next_node_index += 1;
                        observer
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        (
            BlackboardObserverSet {
                schema_id: "benchmark".to_string(),
                observers_by_slot,
                slots_by_node: Vec::new().into_boxed_slice(),
            },
            slots,
        )
    }

    fn legacy_matching(
        observers: &BlackboardObserverSet,
        changed_slots: &[BlackboardSlot],
    ) -> Vec<BlackboardObserver> {
        changed_slots
            .iter()
            .flat_map(|slot| {
                observers
                    .observers_by_slot
                    .get(slot.generation_index() as usize)
                    .into_iter()
                    .flat_map(|observers| observers.iter().copied())
            })
            .collect()
    }

    fn legacy_append_matching(
        observers: &BlackboardObserverSet,
        changed_slots: &[BlackboardSlot],
        matching: &mut Vec<BlackboardObserver>,
    ) {
        let observer_count = changed_slots
            .iter()
            .filter_map(|slot| {
                observers
                    .observers_by_slot
                    .get(slot.generation_index() as usize)
            })
            .map(|observers| observers.len())
            .sum();
        matching.reserve(observer_count);
        for slot in changed_slots {
            if let Some(slot_observers) = observers
                .observers_by_slot
                .get(slot.generation_index() as usize)
            {
                matching.extend_from_slice(slot_observers);
            }
        }
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
