use std::hint::black_box;
use std::time::Instant;

use crate::core::framework::render::{
    RenderCapabilitySummary, RenderDeviceDiagnostics, RenderDeviceLimitDiagnostics, RenderStats,
};

use super::take_runtime_render_diagnostics;

#[test]
fn optimization_batch_du_owned_render_projection_preserves_framework_contract() {
    let stats = RenderStats {
        capabilities: RenderCapabilitySummary {
            backend_name: "wgpu(dx12)".to_owned(),
            ..RenderCapabilitySummary::default()
        },
        device_diagnostics: Some(RenderDeviceDiagnostics {
            adapter_name: "Zircon Test Adapter".to_owned(),
            adapter_device_type: "discrete_gpu".to_owned(),
            limits: RenderDeviceLimitDiagnostics {
                max_bind_groups: 5,
                max_texture_dimension_2d: 16_384,
                max_texture_array_layers: 256,
                max_sampled_textures_per_shader_stage: 16,
                max_binding_array_elements_per_shader_stage: 500_000,
                max_binding_array_sampler_elements_per_shader_stage: 1_000,
                max_storage_buffers_per_shader_stage: 8,
                max_storage_buffer_binding_size: 134_217_728,
            },
        }),
        ..RenderStats::default()
    };

    let (backend_name, snapshot) = take_runtime_render_diagnostics(Some(stats));
    let snapshot = snapshot.expect("device snapshot");

    assert_eq!(backend_name.as_deref(), Some("wgpu(dx12)"));
    assert_eq!(snapshot.adapter_name, "Zircon Test Adapter");
    assert_eq!(snapshot.adapter_device_type, "discrete_gpu");
    assert_eq!(snapshot.max_bind_groups, 5);
    assert_eq!(snapshot.max_texture_dimension_2d, 16_384);
    assert_eq!(snapshot.max_texture_array_layers, 256);
    assert_eq!(snapshot.max_sampled_textures_per_shader_stage, 16);
    assert_eq!(
        snapshot.max_binding_array_elements_per_shader_stage,
        500_000
    );
    assert_eq!(
        snapshot.max_binding_array_sampler_elements_per_shader_stage,
        1_000
    );
    assert_eq!(snapshot.max_storage_buffers_per_shader_stage, 8);
    assert_eq!(snapshot.max_storage_buffer_binding_size, 134_217_728);
}

#[test]
fn optimization_batch_du_owned_render_projection_moves_framework_strings() {
    let stats = render_stats_fixture(64);
    let backend_ptr = stats.capabilities.backend_name.as_ptr();
    let device = stats
        .device_diagnostics
        .as_ref()
        .expect("fixture device diagnostics");
    let adapter_ptr = device.adapter_name.as_ptr();
    let device_type_ptr = device.adapter_device_type.as_ptr();

    let (backend_name, device) = take_runtime_render_diagnostics(Some(stats));
    let backend_name = backend_name.expect("backend name");
    let device = device.expect("device snapshot");

    assert_eq!(backend_name.as_ptr(), backend_ptr);
    assert_eq!(device.adapter_name.as_ptr(), adapter_ptr);
    assert_eq!(device.adapter_device_type.as_ptr(), device_type_ptr);
}

#[test]
fn optimization_batch_du_owned_render_projection_omits_unavailable_facts() {
    assert_eq!(take_runtime_render_diagnostics(None), (None, None));
    assert_eq!(
        take_runtime_render_diagnostics(Some(RenderStats::default())),
        (None, None)
    );
}

#[test]
fn optimization_batch_du_owned_render_projection_avoids_string_clones() {
    let production = include_str!("../diagnostics.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("runtime diagnostics production source");
    let projection = production
        .split("fn take_runtime_render_diagnostics")
        .nth(1)
        .expect("owned render diagnostics projection");

    assert!(projection.contains("stats.capabilities.backend_name"));
    assert!(projection.contains("stats.device_diagnostics"));
    assert!(!projection.contains("backend_name.clone()"));
    assert!(!projection.contains("adapter_name.clone()"));
    assert!(!projection.contains("adapter_device_type.clone()"));
}

#[test]
#[ignore = "release-only alternating p95 performance gate"]
fn optimization_batch_du_owned_render_diagnostics_projection_p95() {
    const SAMPLE_PAIRS: usize = 17;
    const PROJECTIONS_PER_SAMPLE: usize = 256;
    const STRING_BYTES: usize = 4_096;

    let prototype = render_stats_fixture(STRING_BYTES);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_render_projections(
                &prototype,
                PROJECTIONS_PER_SAMPLE,
                false,
            ));
            optimized_samples.push(measure_render_projections(
                &prototype,
                PROJECTIONS_PER_SAMPLE,
                true,
            ));
        } else {
            optimized_samples.push(measure_render_projections(
                &prototype,
                PROJECTIONS_PER_SAMPLE,
                true,
            ));
            legacy_samples.push(measure_render_projections(
                &prototype,
                PROJECTIONS_PER_SAMPLE,
                false,
            ));
        }
    }

    let legacy_p95 = p95(&mut legacy_samples);
    let optimized_p95 = p95(&mut optimized_samples);
    println!(
        "RUNTIME430_OWNED_RENDER_DIAGNOSTICS_PROJECTION_BENCH_V1 projections_per_sample={PROJECTIONS_PER_SAMPLE} string_bytes={STRING_BYTES} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
        optimized_p95 as f64 / legacy_p95.max(1) as f64
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
        "owned render diagnostics projection p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
    );
}

fn render_stats_fixture(string_bytes: usize) -> RenderStats {
    RenderStats {
        capabilities: RenderCapabilitySummary {
            backend_name: "b".repeat(string_bytes),
            ..RenderCapabilitySummary::default()
        },
        device_diagnostics: Some(RenderDeviceDiagnostics {
            adapter_name: "a".repeat(string_bytes),
            adapter_device_type: "d".repeat(string_bytes),
            limits: RenderDeviceLimitDiagnostics {
                max_bind_groups: 5,
                max_texture_dimension_2d: 16_384,
                max_texture_array_layers: 256,
                max_sampled_textures_per_shader_stage: 16,
                max_binding_array_elements_per_shader_stage: 500_000,
                max_binding_array_sampler_elements_per_shader_stage: 1_000,
                max_storage_buffers_per_shader_stage: 8,
                max_storage_buffer_binding_size: 134_217_728,
            },
        }),
        ..RenderStats::default()
    }
}

fn measure_render_projections(
    prototype: &RenderStats,
    projection_count: usize,
    optimized: bool,
) -> u128 {
    let inputs = vec![prototype.clone(); projection_count];
    let started_at = Instant::now();
    let mut checksum = 0_usize;
    for stats in inputs {
        let projection = if optimized {
            take_runtime_render_diagnostics(Some(stats))
        } else {
            legacy_render_diagnostics_projection(&stats)
        };
        checksum = checksum
            .wrapping_add(projection.0.as_ref().map_or(0, String::len))
            .wrapping_add(
                projection
                    .1
                    .as_ref()
                    .map_or(0, |device| device.adapter_name.len()),
            );
        black_box(projection);
    }
    black_box(checksum);
    started_at.elapsed().as_nanos()
}

fn legacy_render_diagnostics_projection(
    stats: &RenderStats,
) -> (
    Option<String>,
    Option<zircon_runtime_interface::RuntimeRenderDeviceDiagnosticsSnapshot>,
) {
    let backend_name =
        Some(stats.capabilities.backend_name.clone()).filter(|name| !name.trim().is_empty());
    let device = stats.device_diagnostics.as_ref().map(|diagnostics| {
        zircon_runtime_interface::RuntimeRenderDeviceDiagnosticsSnapshot {
            adapter_name: diagnostics.adapter_name.clone(),
            adapter_device_type: diagnostics.adapter_device_type.clone(),
            max_bind_groups: diagnostics.limits.max_bind_groups,
            max_texture_dimension_2d: diagnostics.limits.max_texture_dimension_2d,
            max_texture_array_layers: diagnostics.limits.max_texture_array_layers,
            max_sampled_textures_per_shader_stage: diagnostics
                .limits
                .max_sampled_textures_per_shader_stage,
            max_binding_array_elements_per_shader_stage: diagnostics
                .limits
                .max_binding_array_elements_per_shader_stage,
            max_binding_array_sampler_elements_per_shader_stage: diagnostics
                .limits
                .max_binding_array_sampler_elements_per_shader_stage,
            max_storage_buffers_per_shader_stage: diagnostics
                .limits
                .max_storage_buffers_per_shader_stage,
            max_storage_buffer_binding_size: diagnostics.limits.max_storage_buffer_binding_size,
        }
    });
    (backend_name, device)
}

fn p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
}
