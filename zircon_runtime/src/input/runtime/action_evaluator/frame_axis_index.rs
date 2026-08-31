use crate::input::{GamepadAxisInput, GamepadAxisTransition, InputFrameSnapshot};

#[derive(Debug, Default)]
pub(super) struct FrameAxisIndex {
    values: Vec<AxisValue>,
    transitions: Vec<AxisTransition>,
    #[cfg(test)]
    source_visits: usize,
}

#[derive(Clone, Copy, Debug)]
struct AxisValue {
    input: GamepadAxisInput,
    source_index: usize,
    value: f32,
}

#[derive(Clone, Copy, Debug)]
struct AxisTransition {
    input: GamepadAxisInput,
    source_index: usize,
    transition: GamepadAxisTransition,
}

impl FrameAxisIndex {
    pub(super) fn load_frame(&mut self, frame: &InputFrameSnapshot) {
        self.values.clear();
        self.values.extend(
            frame
                .gamepad_axes
                .iter()
                .enumerate()
                .map(|(source_index, state)| AxisValue {
                    input: GamepadAxisInput::new(state.gamepad, state.axis),
                    source_index,
                    value: state.value,
                }),
        );
        self.values
            .sort_unstable_by_key(|value| (value.input, value.source_index));
        retain_latest_value(&mut self.values);

        self.transitions.clear();
        self.transitions
            .extend(frame.gamepad_axis_transitions.iter().enumerate().map(
                |(source_index, transition)| AxisTransition {
                    input: GamepadAxisInput::new(transition.gamepad, transition.axis),
                    source_index,
                    transition: *transition,
                },
            ));
        self.transitions
            .sort_unstable_by_key(|transition| (transition.input, transition.source_index));
        retain_latest_transition(&mut self.transitions);

        #[cfg(test)]
        {
            self.source_visits = frame
                .gamepad_axes
                .len()
                .saturating_add(frame.gamepad_axis_transitions.len());
        }
    }

    pub(super) fn value(&self, axis: GamepadAxisInput) -> Option<f32> {
        self.values
            .binary_search_by_key(&axis, |value| value.input)
            .ok()
            .map(|index| self.values[index].value)
    }

    pub(super) fn transition(&self, axis: GamepadAxisInput) -> Option<GamepadAxisTransition> {
        self.transitions
            .binary_search_by_key(&axis, |transition| transition.input)
            .ok()
            .map(|index| self.transitions[index].transition)
    }

    pub(super) fn clear(&mut self) {
        self.values.clear();
        self.transitions.clear();
        #[cfg(test)]
        {
            self.source_visits = 0;
        }
    }

    pub(super) fn storage_capacity(&self) -> usize {
        self.values
            .capacity()
            .saturating_add(self.transitions.capacity())
    }

    #[cfg(test)]
    pub(super) fn source_visit_count(&self) -> usize {
        self.source_visits
    }
}

fn retain_latest_value(values: &mut Vec<AxisValue>) {
    let mut retained = 0;
    for index in 0..values.len() {
        if retained > 0 && values[retained - 1].input == values[index].input {
            values[retained - 1] = values[index];
        } else {
            if retained != index {
                values.swap(retained, index);
            }
            retained += 1;
        }
    }
    values.truncate(retained);
}

fn retain_latest_transition(transitions: &mut Vec<AxisTransition>) {
    let mut retained = 0;
    for index in 0..transitions.len() {
        if retained > 0 && transitions[retained - 1].input == transitions[index].input {
            transitions[retained - 1] = transitions[index];
        } else {
            if retained != index {
                transitions.swap(retained, index);
            }
            retained += 1;
        }
    }
    transitions.truncate(retained);
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;
    use crate::input::{GamepadAxis, GamepadId};

    const BENCHMARK_AXIS_COUNT: usize = 4_096;
    const BENCHMARK_SAMPLES: usize = 11;
    const BENCHMARK_ITERATIONS: usize = 256;

    #[test]
    fn runtime56_recovery_batch_skipped_frame_axis_self_swaps_preserve_latest_value_and_transition()
    {
        let first_input = axis_input(1);
        let second_input = axis_input(2);
        let mut values = vec![
            axis_value(first_input, 0, 0.25),
            axis_value(first_input, 2, 0.75),
            axis_value(second_input, 1, -0.5),
        ];
        let mut retired_values = values.clone();
        retain_latest_value(&mut values);
        retired_retain_latest_value(&mut retired_values);

        assert_eq!(value_signatures(&values), value_signatures(&retired_values));
        assert_eq!(value_signatures(&values), vec![(1, 2, 0.75), (2, 1, -0.5)]);

        let mut transitions = vec![
            axis_transition(first_input, 0, 0.0, 0.25),
            axis_transition(first_input, 2, 0.25, 0.75),
            axis_transition(second_input, 1, 0.0, -0.5),
        ];
        let mut retired_transitions = transitions.clone();
        retain_latest_transition(&mut transitions);
        retired_retain_latest_transition(&mut retired_transitions);

        assert_eq!(
            transition_signatures(&transitions),
            transition_signatures(&retired_transitions)
        );
        assert_eq!(
            transition_signatures(&transitions),
            vec![(1, 2, 0.25, 0.75), (2, 1, 0.0, -0.5)]
        );
    }

    #[test]
    fn runtime56_recovery_batch_skipped_frame_axis_self_swaps_source_contract() {
        let source = include_str!("frame_axis_index.rs");
        let production = source
            .split_once("\n#[cfg(test)]\nmod tests")
            .expect("production module end")
            .0;
        let value_compaction = production
            .split_once("fn retain_latest_value")
            .expect("value compaction")
            .1
            .split_once("fn retain_latest_transition")
            .expect("value compaction end")
            .0;
        let transition_compaction = production
            .split_once("fn retain_latest_transition")
            .expect("transition compaction")
            .1;

        assert!(value_compaction.contains("if retained != index"));
        assert!(transition_compaction.contains("if retained != index"));
        assert_eq!(
            value_compaction
                .matches("values.swap(retained, index)")
                .count(),
            1
        );
        assert_eq!(
            transition_compaction
                .matches("transitions.swap(retained, index)")
                .count(),
            1
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn runtime56_recovery_batch_skipped_frame_axis_self_swaps_release_benchmark() {
        let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

        for sample in 0..BENCHMARK_SAMPLES {
            if sample % 2 == 0 {
                retired_samples.push(measure_value_compaction(retired_retain_latest_value));
                optimized_samples.push(measure_value_compaction(retain_latest_value));
            } else {
                optimized_samples.push(measure_value_compaction(retain_latest_value));
                retired_samples.push(measure_value_compaction(retired_retain_latest_value));
            }
        }

        let retired_p95 = percentile_95(&mut retired_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        let reduction_basis_points = 10_000_u128.saturating_sub(
            optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
        );
        eprintln!(
            "RUNTIME56_SKIP_FRAME_AXIS_SELF_SWAPS_BENCH_V1 \
samples={BENCHMARK_SAMPLES} iterations={BENCHMARK_ITERATIONS} axes={BENCHMARK_AXIS_COUNT} \
retired_self_swaps_per_compaction=4096 optimized_self_swaps_per_compaction=0 \
retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={reduction_basis_points}",
            retired_p95.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(
            optimized_p95.as_nanos().saturating_mul(100)
                <= retired_p95.as_nanos().saturating_mul(80),
            "skipping frame-axis self swaps must reduce unique-axis compaction P95 by at least 20%: \
retired={retired_p95:?}, optimized={optimized_p95:?}"
        );
    }

    fn axis_input(gamepad: u64) -> GamepadAxisInput {
        GamepadAxisInput::new(GamepadId(gamepad), GamepadAxis::LeftStickX)
    }

    fn axis_value(input: GamepadAxisInput, source_index: usize, value: f32) -> AxisValue {
        AxisValue {
            input,
            source_index,
            value,
        }
    }

    fn axis_transition(
        input: GamepadAxisInput,
        source_index: usize,
        previous_value: f32,
        value: f32,
    ) -> AxisTransition {
        AxisTransition {
            input,
            source_index,
            transition: GamepadAxisTransition {
                gamepad: input.gamepad,
                axis: input.axis,
                previous_value,
                value,
            },
        }
    }

    fn value_signatures(values: &[AxisValue]) -> Vec<(u64, usize, f32)> {
        values
            .iter()
            .map(|value| (value.input.gamepad.0, value.source_index, value.value))
            .collect()
    }

    fn transition_signatures(transitions: &[AxisTransition]) -> Vec<(u64, usize, f32, f32)> {
        transitions
            .iter()
            .map(|transition| {
                (
                    transition.input.gamepad.0,
                    transition.source_index,
                    transition.transition.previous_value,
                    transition.transition.value,
                )
            })
            .collect()
    }

    fn retired_retain_latest_value(values: &mut Vec<AxisValue>) {
        let mut retained = 0;
        for index in 0..values.len() {
            if retained > 0 && values[retained - 1].input == values[index].input {
                values[retained - 1] = values[index];
            } else {
                values.swap(retained, index);
                retained += 1;
            }
        }
        values.truncate(retained);
    }

    fn retired_retain_latest_transition(transitions: &mut Vec<AxisTransition>) {
        let mut retained = 0;
        for index in 0..transitions.len() {
            if retained > 0 && transitions[retained - 1].input == transitions[index].input {
                transitions[retained - 1] = transitions[index];
            } else {
                transitions.swap(retained, index);
                retained += 1;
            }
        }
        transitions.truncate(retained);
    }

    fn measure_value_compaction(mut retain: impl FnMut(&mut Vec<AxisValue>)) -> Duration {
        let mut values = (0..BENCHMARK_AXIS_COUNT)
            .map(|index| axis_value(axis_input(index as u64), index, index as f32))
            .collect::<Vec<_>>();
        black_box(&mut values);

        let started = Instant::now();
        for _ in 0..BENCHMARK_ITERATIONS {
            retain(&mut values);
            black_box(&values);
        }
        started.elapsed()
    }

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
