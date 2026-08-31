use crate::input::{GamepadAxisInput, InputButton};

/// Reusable sorted indices into the caller-owned UI-consumed input slices.
#[derive(Debug, Default)]
pub(super) struct ConsumedInputIndex {
    button_indices: Vec<usize>,
    axis_indices: Vec<usize>,
    #[cfg(test)]
    source_visits: usize,
}

impl ConsumedInputIndex {
    pub(super) fn load(
        &mut self,
        consumed_buttons: &[InputButton],
        consumed_axes: &[GamepadAxisInput],
    ) {
        refresh_sorted_indices(&mut self.button_indices, consumed_buttons);
        refresh_sorted_indices(&mut self.axis_indices, consumed_axes);

        #[cfg(test)]
        {
            self.source_visits = consumed_buttons.len().saturating_add(consumed_axes.len());
        }
    }

    pub(super) fn button_is_consumed(
        &self,
        consumed_buttons: &[InputButton],
        button: &InputButton,
    ) -> bool {
        self.button_indices
            .binary_search_by(|index| consumed_buttons[*index].cmp(button))
            .is_ok()
    }

    pub(super) fn axis_is_consumed(
        &self,
        consumed_axes: &[GamepadAxisInput],
        axis: GamepadAxisInput,
    ) -> bool {
        self.axis_indices
            .binary_search_by(|index| consumed_axes[*index].cmp(&axis))
            .is_ok()
    }

    pub(super) fn clear(&mut self) {
        self.button_indices.clear();
        self.axis_indices.clear();
        #[cfg(test)]
        {
            self.source_visits = 0;
        }
    }

    pub(super) fn storage_capacity(&self) -> usize {
        self.button_indices
            .capacity()
            .saturating_add(self.axis_indices.capacity())
    }

    #[cfg(test)]
    pub(super) fn source_visit_count(&self) -> usize {
        self.source_visits
    }
}

fn refresh_sorted_indices<T: Ord>(indices: &mut Vec<usize>, values: &[T]) {
    if indices.len() == values.len()
        && indices
            .windows(2)
            .all(|pair| values[pair[0]] <= values[pair[1]])
    {
        return;
    }

    indices.clear();
    indices.extend(0..values.len());
    indices.sort_unstable_by(|left, right| values[*left].cmp(&values[*right]));
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;
    use crate::input::{GamepadAxis, GamepadId};

    const BENCHMARK_INPUT_COUNT: usize = 4_096;
    const BENCHMARK_SAMPLES: usize = 11;
    const BENCHMARK_ITERATIONS: usize = 128;

    #[test]
    fn runtime56_recovery_batch_reused_consumed_input_permutation_preserves_queries_and_rebuilds_when_needed(
    ) {
        let mut index = ConsumedInputIndex::default();
        let buttons = vec![
            InputButton::KeyCode(30),
            InputButton::KeyCode(10),
            InputButton::KeyCode(20),
        ];
        let mut axes = vec![axis_input(30), axis_input(10), axis_input(20)];

        index.load(&buttons, &axes);
        assert_eq!(index.button_indices, vec![1, 2, 0]);
        assert_eq!(index.axis_indices, vec![1, 2, 0]);
        assert!(index.button_is_consumed(&buttons, &InputButton::KeyCode(20)));
        assert!(index.axis_is_consumed(&axes, axis_input(20)));
        assert!(!index.button_is_consumed(&buttons, &InputButton::KeyCode(99)));
        assert!(!index.axis_is_consumed(&axes, axis_input(99)));

        let stable_button_indices = index.button_indices.clone();
        let stable_axis_indices = index.axis_indices.clone();
        index.load(&buttons, &axes);
        assert_eq!(index.button_indices, stable_button_indices);
        assert_eq!(index.axis_indices, stable_axis_indices);

        axes.swap(0, 1);
        index.load(&buttons, &axes);
        assert_eq!(index.axis_indices, vec![0, 2, 1]);
        assert!(index.axis_is_consumed(&axes, axis_input(30)));
    }

    #[test]
    fn runtime56_recovery_batch_reused_consumed_input_permutation_source_contract() {
        let source = include_str!("consumed_input_index.rs");
        let production = source
            .split_once("\n#[cfg(test)]\nmod tests")
            .expect("production module end")
            .0;
        let loader = production
            .split_once("fn refresh_sorted_indices")
            .expect("shared sorted-index loader")
            .1;
        let return_position = loader
            .find("return")
            .expect("cached permutation early return");
        let clear_position = loader.find("indices.clear()").expect("rebuild clear");

        assert!(production.contains("fn refresh_sorted_indices<T: Ord>"));
        assert_eq!(production.matches("refresh_sorted_indices(&mut").count(), 2);
        assert!(loader.contains(".windows(2)"));
        assert!(loader.contains(".all(|pair|"));
        assert!(return_position < clear_position);
        assert_eq!(loader.matches("sort_unstable_by").count(), 1);
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn runtime56_recovery_batch_reused_consumed_input_permutation_release_benchmark() {
        let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

        for sample in 0..BENCHMARK_SAMPLES {
            if sample % 2 == 0 {
                retired_samples.push(measure_repeated_load(retired_load_sorted_indices));
                optimized_samples.push(measure_repeated_load(refresh_sorted_indices));
            } else {
                optimized_samples.push(measure_repeated_load(refresh_sorted_indices));
                retired_samples.push(measure_repeated_load(retired_load_sorted_indices));
            }
        }

        let retired_p95 = percentile_95(&mut retired_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        let reduction_basis_points = 10_000_u128.saturating_sub(
            optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
        );
        eprintln!(
            "RUNTIME56_REUSED_CONSUMED_INPUT_PERMUTATION_BENCH_V1 \
samples={BENCHMARK_SAMPLES} iterations={BENCHMARK_ITERATIONS} inputs={BENCHMARK_INPUT_COUNT} \
retired_index_writes_per_reload=4096 optimized_index_writes_per_reload=0 \
retired_sorts_per_reload=1 optimized_sorts_per_reload=0 \
retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={reduction_basis_points}",
            retired_p95.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(
            optimized_p95.as_nanos().saturating_mul(100)
                <= retired_p95.as_nanos().saturating_mul(35),
            "reusing the consumed-input permutation must reduce stable reload P95 by at least 65%: \
retired={retired_p95:?}, optimized={optimized_p95:?}"
        );
    }

    fn axis_input(gamepad: u64) -> GamepadAxisInput {
        GamepadAxisInput::new(GamepadId(gamepad), GamepadAxis::LeftStickX)
    }

    fn retired_load_sorted_indices(indices: &mut Vec<usize>, values: &[GamepadAxisInput]) {
        indices.clear();
        indices.extend(0..values.len());
        indices.sort_unstable_by(|left, right| values[*left].cmp(&values[*right]));
    }

    fn measure_repeated_load(
        mut load: impl FnMut(&mut Vec<usize>, &[GamepadAxisInput]),
    ) -> Duration {
        let values = (0..BENCHMARK_INPUT_COUNT)
            .map(|index| {
                let permuted = index.wrapping_mul(4_053) & (BENCHMARK_INPUT_COUNT - 1);
                axis_input(permuted as u64)
            })
            .collect::<Vec<_>>();
        let mut indices = Vec::new();
        load(&mut indices, &values);
        black_box((&values, &indices));

        let started = Instant::now();
        for _ in 0..BENCHMARK_ITERATIONS {
            load(&mut indices, &values);
            black_box(&indices);
        }
        started.elapsed()
    }

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
