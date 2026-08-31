use serde::{Deserialize, Serialize};

use super::{GamepadAxis, GamepadId, InputButton};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum InputAxisDirection {
    #[default]
    Full,
    Positive,
    Negative,
}

impl InputAxisDirection {
    pub fn value(self, source: f32) -> f32 {
        let value = if source.is_finite() {
            source.clamp(-1.0, 1.0)
        } else {
            0.0
        };

        match self {
            Self::Full => normalized_axis_value(value),
            Self::Positive => normalized_axis_value(value.max(0.0)),
            Self::Negative => normalized_axis_value((-value).max(0.0)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputAxisBinding {
    pub gamepad: GamepadId,
    pub axis: GamepadAxis,
    #[serde(default)]
    pub direction: InputAxisDirection,
}

impl InputAxisBinding {
    pub const fn new(gamepad: GamepadId, axis: GamepadAxis) -> Self {
        Self {
            gamepad,
            axis,
            direction: InputAxisDirection::Full,
        }
    }

    pub const fn positive(gamepad: GamepadId, axis: GamepadAxis) -> Self {
        Self {
            gamepad,
            axis,
            direction: InputAxisDirection::Positive,
        }
    }

    pub const fn negative(gamepad: GamepadId, axis: GamepadAxis) -> Self {
        Self {
            gamepad,
            axis,
            direction: InputAxisDirection::Negative,
        }
    }

    pub fn value(self, source: f32) -> f32 {
        self.direction.value(source)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputBinding {
    pub action: String,
    #[serde(default)]
    pub buttons: Vec<InputButton>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub axes: Vec<InputAxisBinding>,
}

impl InputBinding {
    pub fn button(action: impl Into<String>, button: InputButton) -> Self {
        Self::chord(action, [button])
    }

    pub fn chord(
        action: impl Into<String>,
        buttons: impl IntoIterator<Item = InputButton>,
    ) -> Self {
        let mut buttons = buttons.into_iter().collect::<Vec<_>>();
        buttons.sort_unstable();
        buttons.dedup();
        Self {
            action: action.into(),
            buttons,
            axes: Vec::new(),
        }
    }

    pub fn axis(action: impl Into<String>, axis: InputAxisBinding) -> Self {
        Self::axes(action, [axis])
    }

    pub fn axes(
        action: impl Into<String>,
        axes: impl IntoIterator<Item = InputAxisBinding>,
    ) -> Self {
        Self::buttons_and_axes(action, std::iter::empty(), axes)
    }

    pub fn buttons_and_axes(
        action: impl Into<String>,
        buttons: impl IntoIterator<Item = InputButton>,
        axes: impl IntoIterator<Item = InputAxisBinding>,
    ) -> Self {
        let mut binding = Self::chord(action, buttons);
        let mut axes = axes.into_iter().collect::<Vec<_>>();
        axes.sort_unstable_by(|left, right| {
            left.gamepad
                .cmp(&right.gamepad)
                .then(left.axis.cmp(&right.axis))
                .then(left.direction.cmp(&right.direction))
        });
        axes.dedup();
        binding.axes = axes;
        binding
    }

    pub fn is_empty(&self) -> bool {
        self.buttons.is_empty() && self.axes.is_empty()
    }
}

fn normalized_axis_value(value: f32) -> f32 {
    if value == 0.0 {
        0.0
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;

    const BENCHMARK_SAMPLES: usize = 11;
    const BENCHMARK_ITERATIONS: usize = 32;
    const BENCHMARK_BINDING_COUNT: usize = 4_096;

    #[test]
    fn runtime56_batch_scratch_free_binding_sort_preserves_retired_order_and_deduplication() {
        let buttons = vec![
            InputButton::KeyCode(30),
            InputButton::MouseLeft,
            InputButton::KeyCode(10),
            InputButton::KeyCode(30),
            InputButton::MouseLeft,
        ];
        let axes = vec![
            InputAxisBinding::negative(GamepadId(2), GamepadAxis::RightStickX),
            InputAxisBinding::positive(GamepadId(1), GamepadAxis::LeftStickY),
            InputAxisBinding::new(GamepadId(1), GamepadAxis::LeftStickX),
            InputAxisBinding::positive(GamepadId(1), GamepadAxis::LeftStickY),
        ];

        assert_eq!(
            InputBinding::buttons_and_axes("move", buttons.clone(), axes.clone()),
            retired_buttons_and_axes("move", buttons, axes)
        );
    }

    #[test]
    fn runtime56_batch_scratch_free_binding_sort_source_contract() {
        let source = include_str!("input_binding.rs");
        let production = source
            .split_once("\n#[cfg(test)]\nmod tests")
            .expect("production module end")
            .0;

        assert_eq!(production.matches("buttons.sort_unstable();").count(), 1);
        assert_eq!(production.matches("axes.sort_unstable_by(").count(), 1);
        assert!(!production.contains("buttons.sort();"));
        assert!(!production.contains("axes.sort_by("));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn runtime56_batch_scratch_free_binding_sort_release_benchmark() {
        let buttons = (0..BENCHMARK_BINDING_COUNT)
            .map(|index| {
                let permuted = index.wrapping_mul(4_053) & (BENCHMARK_BINDING_COUNT - 1);
                InputButton::KeyCode(permuted as u32)
            })
            .collect::<Vec<_>>();
        let axes = (0..BENCHMARK_BINDING_COUNT)
            .map(|index| {
                let permuted = index.wrapping_mul(4_053) & (BENCHMARK_BINDING_COUNT - 1);
                InputAxisBinding {
                    gamepad: GamepadId(permuted as u64),
                    axis: GamepadAxis::Other((permuted % 256) as u16),
                    direction: match permuted % 3 {
                        0 => InputAxisDirection::Full,
                        1 => InputAxisDirection::Positive,
                        _ => InputAxisDirection::Negative,
                    },
                }
            })
            .collect::<Vec<_>>();
        let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

        for sample in 0..BENCHMARK_SAMPLES {
            if sample % 2 == 0 {
                retired_samples.push(measure_bindings(&buttons, &axes, retired_buttons_and_axes));
                optimized_samples.push(measure_bindings(
                    &buttons,
                    &axes,
                    InputBinding::buttons_and_axes,
                ));
            } else {
                optimized_samples.push(measure_bindings(
                    &buttons,
                    &axes,
                    InputBinding::buttons_and_axes,
                ));
                retired_samples.push(measure_bindings(&buttons, &axes, retired_buttons_and_axes));
            }
        }

        let retired_p95 = percentile_95(&mut retired_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        let reduction_basis_points = 10_000_u128.saturating_sub(
            optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
        );
        eprintln!(
            "RUNTIME56_SCRATCH_FREE_BINDING_SORT_BENCH_V1 \
samples={BENCHMARK_SAMPLES} iterations={BENCHMARK_ITERATIONS} \
buttons={BENCHMARK_BINDING_COUNT} axes={BENCHMARK_BINDING_COUNT} \
retired_stable_sorts_per_binding=2 optimized_stable_sorts_per_binding=0 \
retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={reduction_basis_points}",
            retired_p95.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(
            optimized_p95.as_nanos().saturating_mul(100)
                <= retired_p95.as_nanos().saturating_mul(80),
            "scratch-free sorting must reduce binding construction P95 by at least 20%: \
retired={retired_p95:?}, optimized={optimized_p95:?}"
        );
    }

    fn retired_buttons_and_axes(
        action: impl Into<String>,
        buttons: impl IntoIterator<Item = InputButton>,
        axes: impl IntoIterator<Item = InputAxisBinding>,
    ) -> InputBinding {
        let mut buttons = buttons.into_iter().collect::<Vec<_>>();
        buttons.sort();
        buttons.dedup();
        let mut axes = axes.into_iter().collect::<Vec<_>>();
        axes.sort_by(|left, right| {
            left.gamepad
                .cmp(&right.gamepad)
                .then(left.axis.cmp(&right.axis))
                .then(left.direction.cmp(&right.direction))
        });
        axes.dedup();
        InputBinding {
            action: action.into(),
            buttons,
            axes,
        }
    }

    fn measure_bindings<F>(buttons: &[InputButton], axes: &[InputAxisBinding], build: F) -> Duration
    where
        F: Fn(&'static str, Vec<InputButton>, Vec<InputAxisBinding>) -> InputBinding,
    {
        let mut inputs = (0..BENCHMARK_ITERATIONS)
            .map(|_| (buttons.to_vec(), axes.to_vec()))
            .collect::<Vec<_>>();
        let started = Instant::now();
        for (buttons, axes) in inputs.drain(..) {
            black_box(build("benchmark", buttons, axes));
        }
        started.elapsed()
    }

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
