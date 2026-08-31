use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use super::{InputAction, InputActionContext, InputBinding};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputActionMap {
    #[serde(default)]
    pub contexts: Vec<InputActionContext>,
    pub actions: Vec<InputAction>,
    pub bindings: Vec<InputBinding>,
}

impl InputActionMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_action(mut self, action: InputAction) -> Self {
        self.add_action(action);
        self
    }

    pub fn with_context(mut self, context: InputActionContext) -> Self {
        self.add_context(context);
        self
    }

    pub fn with_binding(mut self, binding: InputBinding) -> Self {
        self.bind(binding);
        self
    }

    pub fn add_context(&mut self, context: InputActionContext) -> &mut Self {
        if self.has_context(&context.id) {
            return self;
        }
        if !contexts_are_sorted(&self.contexts) {
            self.contexts.sort_by(context_order);
        }
        let insertion_index = self
            .contexts
            .partition_point(|candidate| context_order(candidate, &context).is_lt());
        self.contexts.insert(insertion_index, context);
        self
    }

    pub fn add_action(&mut self, action: InputAction) -> &mut Self {
        if !self.has_action(&action.id) {
            self.actions.push(action);
        }
        self
    }

    pub fn bind(&mut self, binding: InputBinding) -> &mut Self {
        if !binding.is_empty() {
            self.bindings.push(binding);
        }
        self
    }

    pub fn clear_bindings(&mut self, action: impl AsRef<str>) -> &mut Self {
        let action = action.as_ref();
        self.bindings.retain(|binding| binding.action != action);
        self
    }

    pub fn has_action(&self, action: impl AsRef<str>) -> bool {
        let action = action.as_ref();
        self.actions.iter().any(|candidate| candidate.id == action)
    }

    pub fn has_context(&self, context: impl AsRef<str>) -> bool {
        let context = context.as_ref();
        self.contexts
            .iter()
            .any(|candidate| candidate.id == context)
    }

    pub fn context_enabled(&self, context: impl AsRef<str>) -> bool {
        let context = context.as_ref();
        self.contexts
            .iter()
            .find(|candidate| candidate.id == context)
            .map(|candidate| candidate.enabled)
            .unwrap_or(true)
    }

    pub fn bindings_for_action<'a>(
        &'a self,
        action: &'a str,
    ) -> impl Iterator<Item = &'a InputBinding> + 'a {
        self.bindings
            .iter()
            .filter(move |binding| binding.action == action)
    }
}

fn contexts_are_sorted(contexts: &[InputActionContext]) -> bool {
    contexts
        .windows(2)
        .all(|pair| !context_order(&pair[0], &pair[1]).is_gt())
}

fn context_order(left: &InputActionContext, right: &InputActionContext) -> Ordering {
    right
        .priority
        .cmp(&left.priority)
        .then(left.id.cmp(&right.id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime56_batch_incremental_context_ordering_preserves_retired_order_and_duplicates() {
        let additions = vec![
            InputActionContext::new("gameplay").with_priority(10),
            InputActionContext::new("menu").with_priority(100),
            InputActionContext::new("photo").with_priority(100),
            InputActionContext::new("background").with_priority(-5),
            InputActionContext::new("gameplay").with_priority(999),
        ];
        let mut retired = InputActionMap::new();
        let mut optimized = InputActionMap::new();

        for context in additions {
            retired_add_context(&mut retired, context.clone());
            optimized.add_context(context);
        }

        assert_eq!(optimized.contexts, retired.contexts);
        assert_eq!(
            optimized
                .contexts
                .iter()
                .filter(|context| context.id == "gameplay")
                .count(),
            1
        );
    }

    #[test]
    fn runtime56_batch_incremental_context_ordering_repairs_public_unsorted_contexts() {
        let mut map = InputActionMap {
            contexts: vec![
                InputActionContext::new("low").with_priority(-10),
                InputActionContext::new("high").with_priority(100),
                InputActionContext::new("middle").with_priority(20),
            ],
            ..InputActionMap::default()
        };

        map.add_context(InputActionContext::new("peer").with_priority(20));

        assert_eq!(
            map.contexts
                .iter()
                .map(|context| (context.id.as_str(), context.priority))
                .collect::<Vec<_>>(),
            vec![("high", 100), ("middle", 20), ("peer", 20), ("low", -10)]
        );
    }

    #[test]
    fn runtime56_batch_incremental_context_ordering_uses_sorted_insertion() {
        let source = include_str!("input_action_map.rs");
        let implementation = source.split("#[cfg(test)]").next().expect("implementation");
        let add_context = implementation
            .split("pub fn add_context")
            .nth(1)
            .expect("add_context")
            .split("pub fn add_action")
            .next()
            .expect("add_context body");

        assert!(add_context.contains("if !contexts_are_sorted(&self.contexts)"));
        assert!(add_context.contains("partition_point"));
        assert!(add_context.contains("self.contexts.insert"));
        assert!(!add_context.contains("self.contexts.push(context)"));
        assert!(implementation.contains("fn context_order"));
    }

    #[test]
    #[ignore = "release performance benchmark"]
    fn runtime56_batch_incremental_context_ordering_release_benchmark() {
        const SAMPLES: usize = 11;
        const ITERATIONS: usize = 8;
        const CONTEXT_COUNT: usize = 512;
        const RETIRED_FULL_SORTS: usize = CONTEXT_COUNT;
        const OPTIMIZED_FULL_SORTS: usize = 0;

        let contexts = (0..CONTEXT_COUNT)
            .rev()
            .map(|index| {
                InputActionContext::new(format!("context_{index:04}"))
                    .with_priority((index % 32) as i32 - 16)
            })
            .collect::<Vec<_>>();
        let mut retired_samples = Vec::with_capacity(SAMPLES);
        let mut optimized_samples = Vec::with_capacity(SAMPLES);
        for sample in 0..SAMPLES {
            let benchmark = |build: fn(Vec<InputActionContext>) -> Vec<InputActionContext>| {
                let inputs = (0..ITERATIONS)
                    .map(|_| contexts.clone())
                    .collect::<Vec<_>>();
                let started = std::time::Instant::now();
                let outputs = inputs.into_iter().map(build).collect::<Vec<_>>();
                std::hint::black_box(&outputs);
                started.elapsed().as_nanos()
            };

            if sample % 2 == 0 {
                retired_samples.push(benchmark(retired_build_contexts));
                optimized_samples.push(benchmark(optimized_build_contexts));
            } else {
                optimized_samples.push(benchmark(optimized_build_contexts));
                retired_samples.push(benchmark(retired_build_contexts));
            }
        }

        let retired_p95_ns = percentile_95(&mut retired_samples);
        let optimized_p95_ns = percentile_95(&mut optimized_samples);
        let reduction_bps = retired_p95_ns
            .saturating_sub(optimized_p95_ns)
            .saturating_mul(10_000)
            / retired_p95_ns.max(1);
        println!(
            "RUNTIME56_INCREMENTAL_CONTEXT_ORDERING_BENCH_V1 \
             retired_p95_ns={retired_p95_ns} optimized_p95_ns={optimized_p95_ns} \
             reduction_bps={reduction_bps} samples={SAMPLES} iterations={ITERATIONS} \
             contexts={CONTEXT_COUNT} full_sorts={RETIRED_FULL_SORTS}->{OPTIMIZED_FULL_SORTS}"
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= retired_p95_ns.saturating_mul(60),
            "optimized P95 must be at least 40% faster: retired={retired_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn retired_add_context(map: &mut InputActionMap, context: InputActionContext) {
        if !map
            .contexts
            .iter()
            .any(|candidate| candidate.id == context.id)
        {
            map.contexts.push(context);
            map.contexts.sort_by(context_order);
        }
    }

    fn retired_build_contexts(contexts: Vec<InputActionContext>) -> Vec<InputActionContext> {
        let mut map = InputActionMap::new();
        for context in contexts {
            retired_add_context(&mut map, context);
        }
        map.contexts
    }

    fn optimized_build_contexts(contexts: Vec<InputActionContext>) -> Vec<InputActionContext> {
        let mut map = InputActionMap::new();
        for context in contexts {
            map.add_context(context);
        }
        map.contexts
    }

    fn percentile_95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
        samples[index]
    }
}
