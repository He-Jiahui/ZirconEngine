use crate::input::{GamepadAxisInput, InputButton, InputFrameSnapshot};

use super::consumed_input_index::ConsumedInputIndex;
use super::frame_axis_index::FrameAxisIndex;
use super::generation::ActionEvaluationGeneration;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct EvaluatedAction {
    pub(super) pressed: bool,
    pub(super) just_activated: bool,
    pub(super) just_deactivated: bool,
    pub(super) value: f32,
}

/// Reusable evaluator-local state. The default action manager serializes evaluator calls.
#[derive(Debug, Default)]
pub(super) struct ActionEvaluationWorkspace {
    frame_axes: FrameAxisIndex,
    consumed_inputs: ConsumedInputIndex,
    active_contexts: Vec<bool>,
    actions: Vec<EvaluatedAction>,
    #[cfg(test)]
    storage_growth_count: usize,
}

impl ActionEvaluationWorkspace {
    pub(super) fn prepare(
        &mut self,
        generation: &ActionEvaluationGeneration,
        frame: &InputFrameSnapshot,
        active_contexts: &[impl AsRef<str>],
        consumed_buttons: &[InputButton],
        consumed_axes: &[GamepadAxisInput],
    ) {
        self.prepare_actions(generation.actions().len());
        self.prepare_contexts(generation, active_contexts);
        self.prepare_frame_axes(generation, frame);
        self.prepare_consumed_inputs(consumed_buttons, consumed_axes);
    }

    pub(super) fn frame_axes(&self) -> &FrameAxisIndex {
        &self.frame_axes
    }

    pub(super) fn consumed_inputs(&self) -> &ConsumedInputIndex {
        &self.consumed_inputs
    }

    #[cfg(test)]
    pub(super) fn consumed_input_source_visit_count(&self) -> usize {
        self.consumed_inputs.source_visit_count()
    }

    pub(super) fn action(&self, slot: usize) -> EvaluatedAction {
        self.actions[slot]
    }

    pub(super) fn set_action(&mut self, slot: usize, action: EvaluatedAction) {
        self.actions[slot] = action;
    }

    pub(super) fn context_is_active(&self, slot: usize) -> bool {
        self.active_contexts.get(slot).copied().unwrap_or(false)
    }

    pub(super) fn reset(&mut self) {
        self.frame_axes.clear();
        self.consumed_inputs.clear();
        self.active_contexts.clear();
        self.actions.clear();
    }

    fn prepare_actions(&mut self, action_count: usize) {
        self.record_growth(
            self.actions.len() < action_count && self.actions.capacity() < action_count,
        );
        reset_action_storage(&mut self.actions, action_count);
    }

    fn prepare_contexts(
        &mut self,
        generation: &ActionEvaluationGeneration,
        active_contexts: &[impl AsRef<str>],
    ) {
        let context_count = generation.context_count();
        self.record_growth(
            self.active_contexts.len() < context_count
                && self.active_contexts.capacity() < context_count,
        );
        self.active_contexts.resize(context_count, false);
        self.active_contexts.fill(false);
        for context in active_contexts {
            if let Some(slot) = generation.context_slot(context.as_ref()) {
                self.active_contexts[slot] = true;
            }
        }
    }

    fn prepare_frame_axes(
        &mut self,
        generation: &ActionEvaluationGeneration,
        frame: &InputFrameSnapshot,
    ) {
        if !generation.has_axis_bindings() {
            self.frame_axes.clear();
            return;
        }
        let capacity_before = self.frame_axes.storage_capacity();
        self.frame_axes.load_frame(frame);
        self.record_growth(self.frame_axes.storage_capacity() > capacity_before);
    }

    fn prepare_consumed_inputs(
        &mut self,
        consumed_buttons: &[InputButton],
        consumed_axes: &[GamepadAxisInput],
    ) {
        let capacity_before = self.consumed_inputs.storage_capacity();
        self.consumed_inputs.load(consumed_buttons, consumed_axes);
        self.record_growth(self.consumed_inputs.storage_capacity() > capacity_before);
    }

    fn record_growth(&mut self, grew: bool) {
        #[cfg(test)]
        if grew {
            self.storage_growth_count = self.storage_growth_count.saturating_add(1);
        }
        #[cfg(not(test))]
        let _ = grew;
    }

    #[cfg(test)]
    pub(super) fn storage_growth_count(&self) -> usize {
        self.storage_growth_count
    }
}

fn reset_action_storage(actions: &mut Vec<EvaluatedAction>, action_count: usize) {
    let reset_len = actions.len().min(action_count);
    actions.resize(action_count, EvaluatedAction::default());
    actions[..reset_len].fill(EvaluatedAction::default());
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;

    const BENCHMARK_ACTION_COUNT: usize = 65_536;
    const BENCHMARK_SAMPLES: usize = 11;
    const BENCHMARK_ITERATIONS: usize = 128;

    #[test]
    fn runtime56_recovery_batch_growth_aware_action_reset_preserves_grow_shrink_and_reuse_behavior()
    {
        let seed = EvaluatedAction {
            pressed: true,
            just_activated: true,
            just_deactivated: true,
            value: 0.75,
        };
        let mut workspace = ActionEvaluationWorkspace {
            actions: vec![seed; 4],
            ..ActionEvaluationWorkspace::default()
        };
        let mut retired = workspace.actions.clone();

        for action_count in [9, 3, 3, 17, 0] {
            workspace.prepare_actions(action_count);
            retired_prepare_actions(&mut retired, action_count);
            assert_eq!(
                action_signatures(&workspace.actions),
                action_signatures(&retired)
            );
            workspace.actions.fill(seed);
            retired.fill(seed);
        }
    }

    #[test]
    fn runtime56_recovery_batch_growth_aware_action_reset_source_contract() {
        let source = include_str!("workspace.rs");
        let production = source
            .split_once("#[cfg(test)]\nmod tests")
            .expect("production module end")
            .0;
        let prepare_actions = production
            .split_once("fn prepare_actions")
            .expect("prepare actions")
            .1
            .split_once("fn prepare_contexts")
            .expect("prepare actions end")
            .0;
        let reset_storage = production
            .split_once("fn reset_action_storage")
            .expect("reset action storage")
            .1;

        assert!(prepare_actions.contains("reset_action_storage"));
        assert!(reset_storage.contains("let reset_len"));
        assert!(reset_storage.contains("actions[..reset_len].fill"));
        assert!(!reset_storage.contains("actions.fill"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn runtime56_recovery_batch_growth_aware_action_reset_release_benchmark() {
        let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

        for sample in 0..BENCHMARK_SAMPLES {
            if sample % 2 == 0 {
                retired_samples.push(measure_action_reset(retired_prepare_actions));
                optimized_samples.push(measure_action_reset(reset_action_storage));
            } else {
                optimized_samples.push(measure_action_reset(reset_action_storage));
                retired_samples.push(measure_action_reset(retired_prepare_actions));
            }
        }

        let retired_p95 = percentile_95(&mut retired_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        let reduction_basis_points = 10_000_u128.saturating_sub(
            optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
        );
        eprintln!(
            "RUNTIME56_GROWTH_AWARE_ACTION_RESET_BENCH_V1 \
samples={BENCHMARK_SAMPLES} iterations={BENCHMARK_ITERATIONS} \
actions={BENCHMARK_ACTION_COUNT} retired_default_writes_per_fresh_reset=131072 \
optimized_default_writes_per_fresh_reset=65536 retired_p95_ns={} optimized_p95_ns={} \
reduction_basis_points={reduction_basis_points}",
            retired_p95.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(
            optimized_p95.as_nanos().saturating_mul(100)
                <= retired_p95.as_nanos().saturating_mul(75),
            "growth-aware action reset must reduce fresh-reset P95 by at least 25%: \
retired={retired_p95:?}, optimized={optimized_p95:?}"
        );
    }

    fn action_signatures(actions: &[EvaluatedAction]) -> Vec<(bool, bool, bool, u32)> {
        actions
            .iter()
            .map(|action| {
                (
                    action.pressed,
                    action.just_activated,
                    action.just_deactivated,
                    action.value.to_bits(),
                )
            })
            .collect()
    }

    fn retired_prepare_actions(actions: &mut Vec<EvaluatedAction>, action_count: usize) {
        actions.resize(action_count, EvaluatedAction::default());
        actions.fill(EvaluatedAction::default());
    }

    fn measure_action_reset(mut reset: impl FnMut(&mut Vec<EvaluatedAction>, usize)) -> Duration {
        let mut actions = Vec::with_capacity(BENCHMARK_ACTION_COUNT);
        actions.resize(BENCHMARK_ACTION_COUNT, EvaluatedAction::default());
        let started = Instant::now();
        for _ in 0..BENCHMARK_ITERATIONS {
            actions.clear();
            reset(&mut actions, BENCHMARK_ACTION_COUNT);
            black_box(&actions);
        }
        started.elapsed()
    }

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
