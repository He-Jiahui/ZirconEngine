use std::collections::{HashMap, HashSet};

use super::{ViewDescriptor, ViewDescriptorId, ViewInstance, ViewInstanceId};

#[derive(Clone, Debug, Default)]
pub struct ViewRegistry {
    pub(super) descriptors: HashMap<ViewDescriptorId, ViewDescriptor>,
    pub(super) instances: HashMap<ViewInstanceId, ViewInstance>,
    pub(super) single_instance_index: HashMap<ViewDescriptorId, ViewInstanceId>,
    pub(super) counters: HashMap<ViewDescriptorId, usize>,
    pub(super) available_capabilities: HashSet<String>,
}

impl ViewRegistry {
    pub fn set_available_capabilities<I, S>(&mut self, capabilities: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.available_capabilities = capabilities.into_iter().map(Into::into).collect();
    }

    pub fn descriptor_capability_error(&self, descriptor: &ViewDescriptor) -> Option<String> {
        const PREFIX: &str = "view descriptor ";
        const DISABLED_CAPABILITIES: &str = " requires disabled capabilities: ";

        let mut required_capabilities = descriptor.required_capabilities.iter();
        let first_missing = required_capabilities
            .find(|capability| !self.available_capabilities.contains(capability.as_str()))?;
        let remaining_capacity = required_capabilities
            .clone()
            .map(|capability| capability.len().saturating_add(2))
            .sum::<usize>();
        let mut error = String::with_capacity(
            PREFIX
                .len()
                .saturating_add(descriptor.descriptor_id.0.len())
                .saturating_add(DISABLED_CAPABILITIES.len())
                .saturating_add(first_missing.len())
                .saturating_add(remaining_capacity),
        );
        error.push_str(PREFIX);
        error.push_str(&descriptor.descriptor_id.0);
        error.push_str(DISABLED_CAPABILITIES);
        error.push_str(first_missing);
        for capability in required_capabilities {
            if self.available_capabilities.contains(capability) {
                continue;
            }
            error.push_str(", ");
            error.push_str(capability);
        }
        Some(error)
    }
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;
    use crate::ui::workbench::view::{ViewDescriptorId, ViewKind};

    const CAPABILITY_COUNT: usize = 256;
    const BENCHMARK_SAMPLES: usize = 11;
    const BENCHMARK_ITERATIONS: usize = 2_000;

    #[test]
    fn capability_error_preserves_missing_order_and_success_semantics() {
        let descriptor = ViewDescriptor::new(
            ViewDescriptorId::new("editor.capability_probe"),
            ViewKind::ActivityView,
            "Capability Probe",
        )
        .with_required_capabilities([
            "editor.project",
            "runtime.render",
            "editor.native_window",
            "runtime.audio",
        ]);
        let mut registry = ViewRegistry::default();
        registry.set_available_capabilities(["runtime.render", "runtime.audio"]);

        assert_eq!(
            registry.descriptor_capability_error(&descriptor),
            Some(
                "view descriptor editor.capability_probe requires disabled capabilities: \
editor.project, editor.native_window"
                    .to_string()
            )
        );

        registry.set_available_capabilities([
            "editor.project",
            "runtime.render",
            "editor.native_window",
            "runtime.audio",
        ]);
        assert_eq!(registry.descriptor_capability_error(&descriptor), None);
    }

    #[test]
    fn capability_error_builds_one_output_buffer_without_missing_string_clones() {
        let source = include_str!("view_registry.rs");
        let implementation = source
            .split_once("pub fn descriptor_capability_error")
            .expect("capability error function")
            .1
            .split_once("\n    }\n}\n")
            .expect("capability error function end")
            .0;
        let cloned_call = [".clone", "d()"].concat();
        let collected_vec = ["collect::<", "Vec"].concat();

        assert!(
            !implementation.contains(&cloned_call),
            "missing capability diagnostics must borrow descriptor strings"
        );
        assert!(
            !implementation.contains(&collected_vec),
            "missing capability diagnostics must write directly into one output buffer"
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn capability_error_single_buffer_release_benchmark() {
        let descriptor = benchmark_descriptor();
        let registry = ViewRegistry::default();
        let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

        for sample in 0..BENCHMARK_SAMPLES {
            if sample % 2 == 0 {
                retired_samples.push(measure_capability_errors(|| {
                    retired_descriptor_capability_error(&registry, &descriptor)
                }));
                optimized_samples.push(measure_capability_errors(|| {
                    registry.descriptor_capability_error(&descriptor)
                }));
            } else {
                optimized_samples.push(measure_capability_errors(|| {
                    registry.descriptor_capability_error(&descriptor)
                }));
                retired_samples.push(measure_capability_errors(|| {
                    retired_descriptor_capability_error(&registry, &descriptor)
                }));
            }
        }

        let retired_p95 = percentile_95(&mut retired_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        let reduction_basis_points = 10_000_u128.saturating_sub(
            optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
        );
        println!(
            "EDITOR52_CAPABILITY_ERROR_SINGLE_BUFFER_BENCH_V1 \
capabilities={CAPABILITY_COUNT} iterations={BENCHMARK_ITERATIONS} \
retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={} \
structural_allocations_per_error=259->1 missing_string_clones=256->0",
            retired_p95.as_nanos(),
            optimized_p95.as_nanos(),
            reduction_basis_points,
        );
        assert!(
            optimized_p95.as_nanos().saturating_mul(100)
                <= retired_p95.as_nanos().saturating_mul(65),
            "single-buffer capability diagnostics must reduce P95 by at least 35%: \
retired={retired_p95:?}, optimized={optimized_p95:?}"
        );
    }

    fn benchmark_descriptor() -> ViewDescriptor {
        ViewDescriptor::new(
            ViewDescriptorId::new("editor.capability_benchmark"),
            ViewKind::ActivityView,
            "Capability Benchmark",
        )
        .with_required_capabilities(
            (0..CAPABILITY_COUNT)
                .map(|index| format!("editor.capability.{index:04}.{}", "x".repeat(48))),
        )
    }

    fn measure_capability_errors(mut build: impl FnMut() -> Option<String>) -> Duration {
        let started = Instant::now();
        for _ in 0..BENCHMARK_ITERATIONS {
            black_box(build()).expect("benchmark descriptor must remain unavailable");
        }
        started.elapsed()
    }

    fn retired_descriptor_capability_error(
        registry: &ViewRegistry,
        descriptor: &ViewDescriptor,
    ) -> Option<String> {
        let missing = descriptor
            .required_capabilities
            .iter()
            .filter(|capability| !registry.available_capabilities.contains(*capability))
            .cloned()
            .collect::<Vec<_>>();
        (!missing.is_empty()).then(|| {
            format!(
                "view descriptor {} requires disabled capabilities: {}",
                descriptor.descriptor_id.0,
                missing.join(", ")
            )
        })
    }

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
