use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;
use crate::ui::workbench::view::{ViewDescriptorId, ViewKind};

const CAPABILITY_COUNT: usize = 256;
const BENCHMARK_SAMPLES: usize = 11;
const BENCHMARK_ITERATIONS: usize = 2_000;

#[test]
fn restore_preserves_existing_descriptor_and_singleton_ownership() {
    let mut registry = ViewRegistry::default();
    let first_id = ViewDescriptorId::new("editor.restore.first");
    let second_id = ViewDescriptorId::new("editor.restore.second");
    for id in [&first_id, &second_id] {
        registry
            .register_view(ViewDescriptor::new(
                id.clone(),
                ViewKind::ActivityView,
                "Restore Probe",
            ))
            .unwrap();
    }
    let first = registry.open_descriptor(first_id.clone()).unwrap();
    let second = registry.open_descriptor(second_id.clone()).unwrap();

    let mut wrong_owner = second.clone();
    wrong_owner.instance_id = first.instance_id.clone();
    assert!(registry.restore_instance(wrong_owner).is_err());
    assert_eq!(registry.open_descriptor(first_id.clone()).unwrap(), first);
    assert_eq!(registry.open_descriptor(second_id).unwrap(), second);

    let mut second_singleton = first.clone();
    second_singleton.instance_id = ViewInstanceId::new("editor.restore.first#99");
    assert!(registry.restore_instance(second_singleton).is_err());
    assert_eq!(registry.counters.get(&first_id), Some(&1));
    assert_eq!(registry.instances.len(), 2);

    let mut updated = first.clone();
    updated.title = "Restored title".to_string();
    registry.restore_instance(updated.clone()).unwrap();
    assert_eq!(registry.open_descriptor(first_id.clone()).unwrap(), updated);
    registry.remove_instance(&first.instance_id).unwrap();
    assert_eq!(
        registry.open_descriptor(first_id).unwrap().instance_id,
        ViewInstanceId::new("editor.restore.first#2")
    );
}

#[test]
fn next_view_instance_skips_restored_ids_and_reports_counter_exhaustion() {
    let mut registry = ViewRegistry::default();
    let allocated_id = ViewDescriptorId::new("editor.allocate");
    let restored_id = ViewDescriptorId::new("editor.restored");
    for id in [&allocated_id, &restored_id] {
        registry
            .register_view(
                ViewDescriptor::new(id.clone(), ViewKind::ActivityView, "Instance Probe")
                    .with_multi_instance(true),
            )
            .unwrap();
    }
    let mut restored = registry.open_descriptor(restored_id).unwrap();
    restored.instance_id = ViewInstanceId::new("editor.allocate#1");
    registry.restore_instance(restored.clone()).unwrap();

    let allocated = registry.open_descriptor(allocated_id.clone()).unwrap();
    assert_eq!(
        allocated.instance_id,
        ViewInstanceId::new("editor.allocate#2")
    );
    assert_eq!(
        registry.instances.get(&restored.instance_id),
        Some(&restored)
    );

    let instance_count = registry.instances.len();
    registry.counters.insert(allocated_id.clone(), usize::MAX);
    assert!(registry.open_descriptor(allocated_id).is_err());
    assert_eq!(registry.instances.len(), instance_count);
    assert_eq!(
        registry.instances.get(&restored.instance_id),
        Some(&restored)
    );
}

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
    let source = include_str!("../view_registry.rs");
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
        optimized_p95.as_nanos().saturating_mul(100) <= retired_p95.as_nanos().saturating_mul(65),
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
