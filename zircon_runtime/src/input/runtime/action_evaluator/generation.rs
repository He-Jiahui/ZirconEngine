use std::collections::BTreeMap;

use crate::input::InputActionMap;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct CompiledAction {
    pub(super) action_index: usize,
    pub(super) context_slot: Option<usize>,
    binding_start: usize,
    binding_end: usize,
}

impl CompiledAction {
    pub(super) fn binding_indices<'a>(
        &self,
        generation: &'a ActionEvaluationGeneration,
    ) -> &'a [usize] {
        &generation.binding_indices[self.binding_start..self.binding_end]
    }
}

/// Immutable map-change-time lookup data for one persisted action map.
#[derive(Clone, Debug, Default)]
pub(super) struct ActionEvaluationGeneration {
    actions: Vec<CompiledAction>,
    binding_indices: Vec<usize>,
    context_slots: BTreeMap<String, usize>,
    context_enabled: Vec<bool>,
    has_axis_bindings: bool,
    #[cfg(test)]
    source_binding_count: usize,
}

impl ActionEvaluationGeneration {
    pub(super) fn from_action_map(action_map: &InputActionMap) -> Self {
        let mut context_slots = BTreeMap::new();
        let mut context_enabled = Vec::new();
        for context in &action_map.contexts {
            insert_context_slot(
                &mut context_slots,
                &mut context_enabled,
                &context.id,
                context.enabled,
            );
        }
        for action in &action_map.actions {
            if let Some(context) = action.context.as_deref() {
                insert_context_slot(&mut context_slots, &mut context_enabled, context, true);
            }
        }

        let mut bindings_by_action = BTreeMap::<&str, Vec<usize>>::new();
        for (index, binding) in action_map.bindings.iter().enumerate() {
            bindings_by_action
                .entry(binding.action.as_str())
                .or_default()
                .push(index);
        }

        let mut actions = Vec::with_capacity(action_map.actions.len());
        let mut binding_indices = Vec::with_capacity(action_map.bindings.len());
        for (action_index, action) in action_map.actions.iter().enumerate() {
            let binding_start = binding_indices.len();
            if let Some(indices) = bindings_by_action.get(action.id.as_str()) {
                binding_indices.extend(indices.iter().copied());
            }
            let binding_end = binding_indices.len();
            actions.push(CompiledAction {
                action_index,
                context_slot: action
                    .context
                    .as_deref()
                    .and_then(|context| context_slots.get(context).copied()),
                binding_start,
                binding_end,
            });
        }

        Self {
            actions,
            binding_indices,
            context_slots,
            context_enabled,
            has_axis_bindings: action_map
                .bindings
                .iter()
                .any(|binding| !binding.axes.is_empty()),
            #[cfg(test)]
            source_binding_count: action_map.bindings.len(),
        }
    }

    pub(super) fn actions(&self) -> &[CompiledAction] {
        &self.actions
    }

    pub(super) fn has_axis_bindings(&self) -> bool {
        self.has_axis_bindings
    }

    pub(super) fn context_count(&self) -> usize {
        self.context_enabled.len()
    }

    pub(super) fn context_slot(&self, context: &str) -> Option<usize> {
        self.context_slots.get(context).copied()
    }

    pub(super) fn context_enabled(&self, slot: usize) -> bool {
        self.context_enabled.get(slot).copied().unwrap_or(true)
    }

    #[cfg(test)]
    pub(super) fn candidate_count(&self) -> usize {
        self.source_binding_count
    }
}

fn insert_context_slot(
    context_slots: &mut BTreeMap<String, usize>,
    context_enabled: &mut Vec<bool>,
    context: &str,
    enabled: bool,
) {
    if context_slots.contains_key(context) {
        return;
    }
    let slot = context_enabled.len();
    context_slots.insert(context.to_owned(), slot);
    context_enabled.push(enabled);
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;
    use crate::input::{InputAction, InputActionContext, InputBinding, InputButton};

    const BENCHMARK_BINDING_COUNT: usize = 4_096;
    const BENCHMARK_SAMPLES: usize = 11;
    const BENCHMARK_ITERATIONS: usize = 64;

    #[test]
    fn runtime56_recovery_batch_borrowed_binding_index_matches_retired_generation() {
        let action_map = InputActionMap {
            contexts: vec![
                InputActionContext::new("gameplay"),
                InputActionContext::new("menu").with_enabled(false),
            ],
            actions: vec![
                InputAction::new("gameplay.jump").with_context("gameplay"),
                InputAction::new("menu.confirm").with_context("menu"),
                InputAction::new("global.pause"),
            ],
            bindings: vec![
                InputBinding::button("menu.confirm", InputButton::KeyCode(1)),
                InputBinding::button("gameplay.jump", InputButton::KeyCode(2)),
                InputBinding::button("unknown.action", InputButton::KeyCode(3)),
                InputBinding::button("gameplay.jump", InputButton::KeyCode(4)),
            ],
        };

        let retired = retired_from_action_map(&action_map);
        let optimized = ActionEvaluationGeneration::from_action_map(&action_map);

        assert_eq!(optimized.context_slots, retired.context_slots);
        assert_eq!(optimized.context_enabled, retired.context_enabled);
        assert_eq!(optimized.binding_indices, retired.binding_indices);
        assert_eq!(optimized.has_axis_bindings, retired.has_axis_bindings);
        assert_eq!(optimized.source_binding_count, retired.source_binding_count);
        assert_eq!(optimized.actions.len(), retired.actions.len());
        for (optimized_action, retired_action) in optimized.actions.iter().zip(&retired.actions) {
            assert_eq!(optimized_action.action_index, retired_action.action_index);
            assert_eq!(optimized_action.context_slot, retired_action.context_slot);
            assert_eq!(
                optimized_action.binding_indices(&optimized),
                retired_action.binding_indices(&retired)
            );
        }
    }

    #[test]
    fn runtime56_recovery_batch_generation_binding_index_borrows_action_keys() {
        let source = include_str!("generation.rs");
        let implementation = source
            .split_once("pub(super) fn from_action_map")
            .expect("generation builder")
            .1
            .split_once("    pub(super) fn actions")
            .expect("generation builder end")
            .0;
        let cloned_binding_action = ["entry(binding.action", ".clone())"].concat();
        let owned_string_index = ["BTreeMap::<", "String, Vec<usize>>"].concat();

        assert!(
            !implementation.contains(&cloned_binding_action),
            "generation compilation must not clone one action string per binding"
        );
        assert!(
            !implementation.contains(&owned_string_index),
            "the temporary binding index must borrow action strings"
        );
        assert!(implementation.contains("BTreeMap::<&str, Vec<usize>>"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn runtime56_recovery_batch_borrowed_binding_index_release_benchmark() {
        let action_map = benchmark_action_map();
        let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

        for sample in 0..BENCHMARK_SAMPLES {
            if sample % 2 == 0 {
                retired_samples.push(measure_generation(|| retired_from_action_map(&action_map)));
                optimized_samples.push(measure_generation(|| {
                    ActionEvaluationGeneration::from_action_map(&action_map)
                }));
            } else {
                optimized_samples.push(measure_generation(|| {
                    ActionEvaluationGeneration::from_action_map(&action_map)
                }));
                retired_samples.push(measure_generation(|| retired_from_action_map(&action_map)));
            }
        }

        let retired_p95 = percentile_95(&mut retired_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        let reduction_basis_points = 10_000_u128.saturating_sub(
            optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
        );
        eprintln!(
            "RUNTIME56_BORROWED_ACTION_BINDING_INDEX_BENCH_V1 \
samples={BENCHMARK_SAMPLES} iterations={BENCHMARK_ITERATIONS} \
bindings={BENCHMARK_BINDING_COUNT} retired_binding_action_clones_per_build=4096 \
optimized_binding_action_clones_per_build=0 retired_p95_ns={} optimized_p95_ns={} \
reduction_basis_points={reduction_basis_points}",
            retired_p95.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(
            optimized_p95.as_nanos().saturating_mul(100)
                <= retired_p95.as_nanos().saturating_mul(85),
            "borrowed action binding index must reduce generation-build P95 by at least 15%: \
retired={retired_p95:?}, optimized={optimized_p95:?}"
        );
    }

    fn benchmark_action_map() -> InputActionMap {
        let mut actions = Vec::with_capacity(BENCHMARK_BINDING_COUNT);
        let mut bindings = Vec::with_capacity(BENCHMARK_BINDING_COUNT);
        for index in 0..BENCHMARK_BINDING_COUNT {
            let action = format!("gameplay.action.{index:04}.{}", "x".repeat(96));
            actions.push(InputAction::new(action.clone()));
            bindings.push(InputBinding::button(
                action,
                InputButton::KeyCode(index as u32),
            ));
        }
        InputActionMap {
            contexts: Vec::new(),
            actions,
            bindings,
        }
    }

    fn measure_generation(mut build: impl FnMut() -> ActionEvaluationGeneration) -> Duration {
        let started = Instant::now();
        for _ in 0..BENCHMARK_ITERATIONS {
            black_box(build());
        }
        started.elapsed()
    }

    fn retired_from_action_map(action_map: &InputActionMap) -> ActionEvaluationGeneration {
        let mut context_slots = BTreeMap::new();
        let mut context_enabled = Vec::new();
        for context in &action_map.contexts {
            insert_context_slot(
                &mut context_slots,
                &mut context_enabled,
                &context.id,
                context.enabled,
            );
        }
        for action in &action_map.actions {
            if let Some(context) = action.context.as_deref() {
                insert_context_slot(&mut context_slots, &mut context_enabled, context, true);
            }
        }

        let mut bindings_by_action = BTreeMap::<String, Vec<usize>>::new();
        for (index, binding) in action_map.bindings.iter().enumerate() {
            bindings_by_action
                .entry(binding.action.clone())
                .or_default()
                .push(index);
        }

        finish_generation(
            action_map,
            context_slots,
            context_enabled,
            &bindings_by_action,
        )
    }

    fn finish_generation<K>(
        action_map: &InputActionMap,
        context_slots: BTreeMap<String, usize>,
        context_enabled: Vec<bool>,
        bindings_by_action: &BTreeMap<K, Vec<usize>>,
    ) -> ActionEvaluationGeneration
    where
        K: Borrow<str> + Ord,
    {
        let mut actions = Vec::with_capacity(action_map.actions.len());
        let mut binding_indices = Vec::with_capacity(action_map.bindings.len());
        for (action_index, action) in action_map.actions.iter().enumerate() {
            let binding_start = binding_indices.len();
            if let Some(indices) = bindings_by_action.get(action.id.as_str()) {
                binding_indices.extend(indices.iter().copied());
            }
            let binding_end = binding_indices.len();
            actions.push(CompiledAction {
                action_index,
                context_slot: action
                    .context
                    .as_deref()
                    .and_then(|context| context_slots.get(context).copied()),
                binding_start,
                binding_end,
            });
        }

        ActionEvaluationGeneration {
            actions,
            binding_indices,
            context_slots,
            context_enabled,
            has_axis_bindings: action_map
                .bindings
                .iter()
                .any(|binding| !binding.axes.is_empty()),
            source_binding_count: action_map.bindings.len(),
        }
    }

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
