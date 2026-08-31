use zircon_runtime::core::framework::sound::{
    SoundAutomationBinding, SoundAutomationBindingId, SoundAutomationCurve, SoundAutomationTarget,
    SoundError, SoundParameterId,
};

use crate::automation::binding::normalized_automation_binding;
use crate::automation::curve::sample_automation_curve;
use crate::automation::target::apply_automation_target;
use crate::engine::SoundEngineState;

use super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(super) fn bind_automation_impl(
        &self,
        binding: SoundAutomationBinding,
    ) -> Result<(), SoundError> {
        let binding = normalized_automation_binding(binding)?;
        crate::poison_recovery::lock_recover(&self.state)
            .automation_bindings
            .insert(binding.id, binding);
        Ok(())
    }

    pub(super) fn apply_automation_value_impl(
        &self,
        binding: SoundAutomationBindingId,
        value: f32,
    ) -> Result<(), SoundError> {
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        let (target, parameter) = owned_automation_target(&state, binding)?;
        apply_automation_target(&mut state, target, &parameter, value)
    }

    pub(super) fn apply_automation_curve_sample_impl(
        &self,
        binding: SoundAutomationBindingId,
        curve: &SoundAutomationCurve,
        time_seconds: f32,
    ) -> Result<f32, SoundError> {
        let value = sample_automation_curve(curve, time_seconds)?;
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        let (target, parameter) = owned_automation_target(&state, binding)?;
        apply_automation_target(&mut state, target, &parameter, value)?;
        Ok(value)
    }

    pub(super) fn unbind_automation_impl(
        &self,
        binding: SoundAutomationBindingId,
    ) -> Result<(), SoundError> {
        crate::poison_recovery::lock_recover(&self.state)
            .automation_bindings
            .remove(&binding)
            .map(|_| ())
            .ok_or(SoundError::UnknownAutomationBinding { binding })
    }
}

fn owned_automation_target(
    state: &SoundEngineState,
    binding: SoundAutomationBindingId,
) -> Result<(SoundAutomationTarget, SoundParameterId), SoundError> {
    state
        .automation_bindings
        .get(&binding)
        .map(|descriptor| (descriptor.target.clone(), descriptor.parameter.clone()))
        .ok_or(SoundError::UnknownAutomationBinding { binding })
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use zircon_runtime::core::framework::sound::{SoundAutomationKeyframe, SoundAutomationTarget};

    use super::*;

    const BENCHMARK_SAMPLE_PAIRS: usize = 21;
    const BENCHMARK_APPLICATIONS_PER_SAMPLE: usize = 4_096;
    const BENCHMARK_UNUSED_PATH_BYTES: usize = 256;

    #[test]
    fn automation_target_projection_preserves_value_and_curve_application() {
        let (legacy, binding, parameter) = manager_with_binding(64);
        let (optimized, optimized_binding, optimized_parameter) = manager_with_binding(64);
        let curve = SoundAutomationCurve::from_keyframes([
            SoundAutomationKeyframe::linear(0.0, 0.25),
            SoundAutomationKeyframe::linear(1.0, 0.75),
        ]);

        legacy_apply_automation_value(&legacy, binding, 0.4).unwrap();
        optimized
            .apply_automation_value_impl(optimized_binding, 0.4)
            .unwrap();
        let legacy_curve_value = sample_automation_curve(&curve, 0.6).unwrap();
        legacy_apply_automation_value(&legacy, binding, legacy_curve_value).unwrap();
        let optimized_curve_value = optimized
            .apply_automation_curve_sample_impl(optimized_binding, &curve, 0.6)
            .unwrap();

        assert_eq!(optimized_curve_value, legacy_curve_value);
        let legacy_state = crate::poison_recovery::lock_recover(&legacy.state);
        let optimized_state = crate::poison_recovery::lock_recover(&optimized.state);
        assert_eq!(
            optimized_state.parameters.get(&optimized_parameter),
            legacy_state.parameters.get(&parameter)
        );
        assert_eq!(
            optimized_state.parameters.len(),
            legacy_state.parameters.len()
        );
    }

    #[test]
    #[ignore = "release-only performance gate"]
    fn automation_target_projection_release_gate() {
        black_box(legacy_benchmark_sample());
        black_box(optimized_benchmark_sample());

        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
        for sample_index in 0..BENCHMARK_SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(legacy_benchmark_sample());
                optimized_samples.push(optimized_benchmark_sample());
            } else {
                optimized_samples.push(optimized_benchmark_sample());
                legacy_samples.push(legacy_benchmark_sample());
            }
        }

        let legacy_p50 = nearest_rank_percentile(&legacy_samples, 50);
        let legacy_p95 = nearest_rank_percentile(&legacy_samples, 95);
        let optimized_p50 = nearest_rank_percentile(&optimized_samples, 50);
        let optimized_p95 = nearest_rank_percentile(&optimized_samples, 95);
        let legacy_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_ns = benchmark_samples_csv(&optimized_samples);
        let unused_path_clone_bytes =
            BENCHMARK_APPLICATIONS_PER_SAMPLE * BENCHMARK_UNUSED_PATH_BYTES;

        println!(
            "PERF_RESULT task=plugins11_automation_target_projection applications_per_sample={BENCHMARK_APPLICATIONS_PER_SAMPLE} binding_path_bytes={BENCHMARK_UNUSED_PATH_BYTES} sample_pairs={BENCHMARK_SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank legacy_unused_path_clone_allocations={BENCHMARK_APPLICATIONS_PER_SAMPLE} optimized_unused_path_clone_allocations=0 legacy_unused_path_clone_bytes={unused_path_clone_bytes} optimized_unused_path_clone_bytes=0 threshold_percent=15 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_raw_ns={legacy_ns} optimized_raw_ns={optimized_ns}"
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(85),
            "optimized P95 {optimized_p95}ns must be at least 15% faster than legacy P95 {legacy_p95}ns"
        );
    }

    fn manager_with_binding(
        path_bytes: usize,
    ) -> (
        DefaultSoundManager,
        SoundAutomationBindingId,
        SoundParameterId,
    ) {
        const PATH_PREFIX: &str = "Timeline/";
        const PATH_SUFFIX: &str = ":sound.value";
        assert!(path_bytes >= PATH_PREFIX.len() + PATH_SUFFIX.len());
        let binding = SoundAutomationBindingId::new(1);
        let parameter = SoundParameterId::new("automation.target.projection");
        let manager = DefaultSoundManager::default();
        manager
            .bind_automation_impl(SoundAutomationBinding {
                id: binding,
                timeline_track_path: format!(
                    "{PATH_PREFIX}{}{PATH_SUFFIX}",
                    "p".repeat(path_bytes - PATH_PREFIX.len() - PATH_SUFFIX.len())
                ),
                target: SoundAutomationTarget::SynthParameter(parameter.clone()),
                parameter: SoundParameterId::new("value"),
            })
            .unwrap();
        assert_eq!(
            crate::poison_recovery::lock_recover(&manager.state)
                .automation_bindings
                .get(&binding)
                .unwrap()
                .timeline_track_path
                .len(),
            path_bytes
        );
        (manager, binding, parameter)
    }

    fn legacy_apply_automation_value(
        manager: &DefaultSoundManager,
        binding: SoundAutomationBindingId,
        value: f32,
    ) -> Result<(), SoundError> {
        let mut state = crate::poison_recovery::lock_recover(&manager.state);
        let binding_descriptor = state
            .automation_bindings
            .get(&binding)
            .cloned()
            .ok_or(SoundError::UnknownAutomationBinding { binding })?;
        apply_automation_target(
            &mut state,
            binding_descriptor.target,
            &binding_descriptor.parameter,
            value,
        )
    }

    fn legacy_benchmark_sample() -> u128 {
        benchmark_sample(legacy_apply_automation_value)
    }

    fn optimized_benchmark_sample() -> u128 {
        benchmark_sample(DefaultSoundManager::apply_automation_value_impl)
    }

    fn benchmark_sample(
        operation: fn(
            &DefaultSoundManager,
            SoundAutomationBindingId,
            f32,
        ) -> Result<(), SoundError>,
    ) -> u128 {
        let (manager, binding, parameter) = manager_with_binding(BENCHMARK_UNUSED_PATH_BYTES);
        let started = Instant::now();
        for index in 0..BENCHMARK_APPLICATIONS_PER_SAMPLE {
            operation(
                black_box(&manager),
                binding,
                black_box((index % 101) as f32 / 100.0),
            )
            .unwrap();
        }
        let elapsed = started.elapsed().as_nanos();
        assert!(crate::poison_recovery::lock_recover(&manager.state)
            .parameters
            .contains_key(&parameter));
        elapsed
    }

    fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        assert!(!sorted.is_empty());
        assert!((1..=100).contains(&percentile));
        sorted[(sorted.len() * percentile).div_ceil(100) - 1]
    }

    fn benchmark_samples_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
