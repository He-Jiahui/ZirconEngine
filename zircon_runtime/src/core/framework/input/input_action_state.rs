use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct InputActionState {
    pressed: BTreeSet<String>,
    just_activated: BTreeSet<String>,
    just_deactivated: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    values: BTreeMap<String, f32>,
}

impl InputActionState {
    pub fn from_sets(
        pressed: BTreeSet<String>,
        just_activated: BTreeSet<String>,
        just_deactivated: BTreeSet<String>,
    ) -> Self {
        let values = pressed
            .iter()
            .map(|action| (action.clone(), 1.0))
            .collect::<BTreeMap<_, _>>();
        Self {
            pressed,
            just_activated,
            just_deactivated,
            values,
        }
    }

    pub fn from_sets_and_values(
        pressed: BTreeSet<String>,
        just_activated: BTreeSet<String>,
        just_deactivated: BTreeSet<String>,
        values: BTreeMap<String, f32>,
    ) -> Self {
        Self {
            pressed,
            just_activated,
            just_deactivated,
            values: normalized_action_values(values),
        }
    }

    pub fn pressed(&self, action: impl AsRef<str>) -> bool {
        self.pressed.contains(action.as_ref())
    }

    pub fn just_activated(&self, action: impl AsRef<str>) -> bool {
        self.just_activated.contains(action.as_ref())
    }

    pub fn just_deactivated(&self, action: impl AsRef<str>) -> bool {
        self.just_deactivated.contains(action.as_ref())
    }

    pub fn value(&self, action: impl AsRef<str>) -> f32 {
        self.values.get(action.as_ref()).copied().unwrap_or(0.0)
    }

    pub fn pressed_actions(&self) -> Vec<String> {
        self.pressed.iter().cloned().collect()
    }

    pub fn just_activated_actions(&self) -> Vec<String> {
        self.just_activated.iter().cloned().collect()
    }

    pub fn just_deactivated_actions(&self) -> Vec<String> {
        self.just_deactivated.iter().cloned().collect()
    }

    pub fn valued_actions(&self) -> Vec<(String, f32)> {
        self.values
            .iter()
            .map(|(action, value)| (action.clone(), *value))
            .collect()
    }
}

fn normalized_action_value(value: f32) -> Option<f32> {
    if !value.is_finite() || value == 0.0 {
        None
    } else {
        Some(value.clamp(-1.0, 1.0))
    }
}

fn normalized_action_values(mut values: BTreeMap<String, f32>) -> BTreeMap<String, f32> {
    values.retain(|_, value| {
        let Some(normalized) = normalized_action_value(*value) else {
            return false;
        };
        *value = normalized;
        true
    });
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime56_batch_in_place_action_value_normalization_preserves_retired_values() {
        let values = BTreeMap::from([
            ("positive".to_string(), 0.75),
            ("negative".to_string(), -0.25),
            ("high".to_string(), 4.0),
            ("low".to_string(), -3.0),
            ("zero".to_string(), 0.0),
            ("negative_zero".to_string(), -0.0),
            ("nan".to_string(), f32::NAN),
            ("infinity".to_string(), f32::INFINITY),
        ]);

        assert_eq!(
            normalized_action_values(values.clone()),
            retired_normalized_action_values(values)
        );
    }

    #[test]
    fn runtime56_batch_in_place_action_value_normalization_reuses_input_tree() {
        let source = include_str!("input_action_state.rs");
        let implementation = source.split("#[cfg(test)]").next().expect("implementation");

        assert!(implementation.contains("fn normalized_action_values"));
        assert!(implementation.contains("values.retain("));
        assert!(implementation.contains("*value = normalized"));
        assert!(!implementation.contains(".filter_map(|(action, value)|"));
        assert!(!implementation.contains("values: values\n                .into_iter()"));
    }

    #[test]
    #[ignore = "release performance benchmark"]
    fn runtime56_batch_in_place_action_value_normalization_release_benchmark() {
        const SAMPLES: usize = 11;
        const ITERATIONS: usize = 32;
        const ACTION_COUNT: usize = 2_048;
        const RETIRED_REPLACEMENT_TREES: usize = 1;
        const OPTIMIZED_REPLACEMENT_TREES: usize = 0;

        let base = (0..ACTION_COUNT)
            .map(|index| {
                let value = match index % 8 {
                    0 => 0.0,
                    1 => f32::NAN,
                    2 => 2.0,
                    3 => -2.0,
                    _ => (index % 100) as f32 / 100.0,
                };
                (format!("action_{index:04}"), value)
            })
            .collect::<BTreeMap<_, _>>();
        let retained_actions = base
            .values()
            .filter(|value| normalized_action_value(**value).is_some())
            .count();
        let mut retired_samples = Vec::with_capacity(SAMPLES);
        let mut optimized_samples = Vec::with_capacity(SAMPLES);
        for sample in 0..SAMPLES {
            let benchmark = |normalize: fn(BTreeMap<String, f32>) -> BTreeMap<String, f32>| {
                let inputs = (0..ITERATIONS).map(|_| base.clone()).collect::<Vec<_>>();
                let started = std::time::Instant::now();
                let outputs = inputs.into_iter().map(normalize).collect::<Vec<_>>();
                std::hint::black_box(&outputs);
                started.elapsed().as_nanos()
            };

            if sample % 2 == 0 {
                retired_samples.push(benchmark(retired_normalized_action_values));
                optimized_samples.push(benchmark(normalized_action_values));
            } else {
                optimized_samples.push(benchmark(normalized_action_values));
                retired_samples.push(benchmark(retired_normalized_action_values));
            }
        }

        let retired_p95_ns = percentile_95(&mut retired_samples);
        let optimized_p95_ns = percentile_95(&mut optimized_samples);
        let reduction_bps = retired_p95_ns
            .saturating_sub(optimized_p95_ns)
            .saturating_mul(10_000)
            / retired_p95_ns.max(1);
        println!(
            "RUNTIME56_IN_PLACE_ACTION_VALUE_NORMALIZATION_BENCH_V1 \
             retired_p95_ns={retired_p95_ns} optimized_p95_ns={optimized_p95_ns} \
             reduction_bps={reduction_bps} samples={SAMPLES} iterations={ITERATIONS} \
             actions={ACTION_COUNT} retained_actions={retained_actions} \
             replacement_trees={RETIRED_REPLACEMENT_TREES}->{OPTIMIZED_REPLACEMENT_TREES} \
             retained_key_moves={retained_actions}->0"
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= retired_p95_ns.saturating_mul(65),
            "optimized P95 must be at least 35% faster: retired={retired_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn retired_normalized_action_values(values: BTreeMap<String, f32>) -> BTreeMap<String, f32> {
        values
            .into_iter()
            .filter_map(|(action, value)| {
                normalized_action_value(value).map(|value| (action, value))
            })
            .collect()
    }

    fn percentile_95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
        samples[index]
    }
}
