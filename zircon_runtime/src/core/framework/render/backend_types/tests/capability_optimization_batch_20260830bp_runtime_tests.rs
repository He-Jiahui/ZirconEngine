use std::hint::black_box;
use std::time::Instant;

use super::{
    RenderCapabilityClass, RenderCapabilityKind, RenderCapabilityMismatchDetail,
    RenderCapabilitySummary,
};

const PERF_MARKER: &str = "RUNTIME368_CAPABILITY_REPORT_CAPACITY_BENCH_V1";

#[test]
fn capability_class_report_preserves_membership_and_order() {
    let capabilities = RenderCapabilitySummary {
        virtual_geometry_supported: true,
        supports_storage_buffers: true,
        supports_indirect_draw: true,
        supports_buffer_readback: true,
        ..RenderCapabilitySummary::default()
    };

    let report = capabilities.capability_class_report(RenderCapabilityClass::Advanced);
    assert_eq!(
        report.satisfied,
        vec![
            RenderCapabilityKind::VirtualGeometry,
            RenderCapabilityKind::StorageBuffers,
            RenderCapabilityKind::IndirectDraw,
            RenderCapabilityKind::BufferReadback,
        ]
    );
    assert_eq!(
        report.missing,
        vec![
            RenderCapabilityMismatchDetail::new(RenderCapabilityKind::HybridGlobalIllumination),
            RenderCapabilityMismatchDetail::new(RenderCapabilityKind::AsyncCompute),
            RenderCapabilityMismatchDetail::new(RenderCapabilityKind::AsyncCopy),
        ]
    );
    assert_eq!(
        report.satisfied.len() + report.missing.len(),
        RenderCapabilityClass::Advanced.capability_count(),
    );
    assert_eq!(
        report.satisfied.capacity(),
        RenderCapabilityClass::Advanced.capability_count(),
    );
    assert_eq!(
        report.missing.capacity(),
        RenderCapabilityClass::Advanced.capability_count(),
    );
}

#[test]
fn capability_class_report_uses_class_capacity_hint() {
    let source = include_str!("../capability.rs");
    let production = source.split("#[cfg(test)]").next().expect("implementation");
    assert!(production.contains("let capacity = class.capability_count();"));
    assert!(production.contains("Vec::with_capacity(capacity)"));
    assert!(!production.contains("let mut satisfied = Vec::new();"));
    assert!(!production.contains("let mut missing = Vec::new();"));
}

#[test]
#[ignore = "release performance evidence"]
fn capability_class_report_capacity_p95() {
    const MATCHES: usize = 250_000;
    const SAMPLES: usize = 17;
    let capabilities = black_box(RenderCapabilitySummary {
        virtual_geometry_supported: true,
        hybrid_global_illumination_supported: true,
        acceleration_structures_supported: true,
        inline_ray_query: true,
        ray_tracing_pipeline: true,
        supports_buffer_binding_array: true,
        supports_texture_binding_array: true,
        supports_fxaa: true,
        supports_storage_buffers: true,
        supports_indirect_draw: true,
        supports_buffer_readback: true,
        supports_async_compute: true,
        supports_async_copy: true,
        ..RenderCapabilitySummary::default()
    });
    let mut baseline = Vec::with_capacity(SAMPLES);
    let mut candidate = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        let order = if sample % 2 == 0 { [0, 1] } else { [1, 0] };
        for pass in order {
            let started = Instant::now();
            let mut checksum = 0usize;
            for _ in 0..MATCHES {
                let report = if pass == 0 {
                    let mut satisfied = Vec::new();
                    let mut missing = Vec::new();
                    for capability in RenderCapabilityKind::ALL {
                        if capability.capability_class() != RenderCapabilityClass::Experimental {
                            continue;
                        }
                        if capability.is_satisfied_by(&capabilities) {
                            satisfied.push(capability);
                        } else {
                            missing.push(RenderCapabilityMismatchDetail::new(capability));
                        }
                    }
                    (satisfied, missing)
                } else {
                    let report =
                        capabilities.capability_class_report(RenderCapabilityClass::Experimental);
                    (report.satisfied, report.missing)
                };
                checksum = checksum
                    .wrapping_add(report.0.len())
                    .wrapping_add(report.1.len());
            }
            black_box(checksum);
            let elapsed = started.elapsed().as_nanos();
            if pass == 0 {
                baseline.push(elapsed);
            } else {
                candidate.push(elapsed);
            }
        }
    }
    baseline.sort_unstable();
    candidate.sort_unstable();
    let baseline_p95 = baseline[(SAMPLES * 95).div_ceil(100) - 1];
    let candidate_p95 = candidate[(SAMPLES * 95).div_ceil(100) - 1];
    let reduction =
        100.0 * baseline_p95.saturating_sub(candidate_p95) as f64 / baseline_p95.max(1) as f64;
    println!(
        "{PERF_MARKER} matches={MATCHES} samples={SAMPLES} baseline_p95_ns={baseline_p95} candidate_p95_ns={candidate_p95} p95_reduction_percent={reduction:.2}"
    );
    assert!(candidate_p95.saturating_mul(10) <= baseline_p95.saturating_mul(7));
}
