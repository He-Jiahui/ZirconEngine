use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::diagnostics::{
    RuntimeDiagnosticsSnapshot, RuntimePhysicsBackendDiagnostics, RuntimePhysicsDiagnostics,
    RuntimeRenderDiagnostics,
};
use zircon_runtime::core::framework::render::{
    RenderHybridGiFallbackReason, RenderHybridGiMode, RenderHybridGiProfile, RenderHybridGiQuality,
    RenderHybridGiResolvedSettings, RenderStats,
};

use super::{detail_item_capacity, detail_items, physics_state_display, physics_status};

#[test]
fn runtime_diagnostics_detail_capacity_preserves_item_order_and_bound() {
    let empty = RuntimeDiagnosticsSnapshot::default();
    let empty_items = detail_items(&empty);
    assert_eq!(empty_items.len(), 1);
    assert_eq!(detail_item_capacity(&empty), 1);
    assert!(empty_items.capacity() >= detail_item_capacity(&empty));

    let mut full = RuntimeDiagnosticsSnapshot {
        render: RuntimeRenderDiagnostics {
            stats: Some(RenderStats {
                last_hybrid_gi_resolved_settings: Some(RenderHybridGiResolvedSettings {
                    mode: RenderHybridGiMode::DynamicOnly,
                    profile: RenderHybridGiProfile::IndoorStatic,
                    quality: RenderHybridGiQuality::High,
                    trace_budget: 0,
                    card_budget: 0,
                    voxel_budget: 0,
                    fallback_reason: None,
                }),
                ..RenderStats::default()
            }),
            error: Some("render failed".to_string()),
            ..RuntimeRenderDiagnostics::default()
        },
        ..RuntimeDiagnosticsSnapshot::default()
    };
    full.physics.error = Some("physics failed".to_string());
    full.animation.error = Some("animation failed".to_string());
    full.profile.feature_enabled = true;

    let items = detail_items(&full);
    assert_eq!(items.len(), 11);
    assert_eq!(detail_item_capacity(&full), 11);
    assert!(items.capacity() >= detail_item_capacity(&full));
    assert!(items[0].starts_with("Virtual Geometry Debug:"));
    assert!(items[1].starts_with("Hybrid GI active probes:"));
    assert!(items[5].starts_with("Virtual Geometry visible clusters:"));
    assert_eq!(items[6], "Render error: render failed");
    assert_eq!(items[10], "Profiling over-budget frames: 0");
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor818_runtime_diagnostics_detail_capacity_release_benchmark() {
    const RUNS_PER_SAMPLE: usize = 4_096;
    const SAMPLE_PAIRS: usize = 17;

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_detail_projection(RUNS_PER_SAMPLE, false));
            optimized_samples.push(measure_detail_projection(RUNS_PER_SAMPLE, true));
        } else {
            optimized_samples.push(measure_detail_projection(RUNS_PER_SAMPLE, true));
            legacy_samples.push(measure_detail_projection(RUNS_PER_SAMPLE, false));
        }
    }

    let legacy_growth_events = growth_events(11);
    let optimized_growth_events = 0;
    assert!(legacy_growth_events > optimized_growth_events);
    println!(
        "EDITOR818_RUNTIME_DIAGNOSTICS_DETAIL_CAPACITY_BENCH_V1 runs_per_sample={RUNS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_samples, 95),
        percentile(&optimized_samples, 95),
    );
}

fn measure_detail_projection(runs: usize, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..runs {
        let mut items = if optimized {
            Vec::with_capacity(11)
        } else {
            Vec::new()
        };
        for index in 0..10 {
            items.push(format!("detail-{index}"));
        }
        checksum = checksum.wrapping_add(items.len());
        black_box(items);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn growth_events(item_count: usize) -> usize {
    let mut capacity = 0usize;
    let mut events = 0usize;
    for length in 1..=item_count {
        if length > capacity {
            capacity = if capacity == 0 {
                4
            } else {
                capacity.saturating_mul(2)
            };
            events += 1;
        }
    }
    events
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

#[test]
fn physics_state_display_normalizes_dynamic_backend_text_without_debug_quotes() {
    for (state, expected) in [
        (Some("ready"), "Ready"),
        (Some("disabled"), "Disabled"),
        (Some("unavailable"), "Unavailable"),
        (Some("custom backend state"), "Custom backend state"),
        (Some("   "), "Unknown"),
        (None, "Unknown"),
    ] {
        assert_eq!(physics_state_display(state), expected);
    }
}

#[test]
fn physics_status_projects_human_readable_dynamic_backend_state() {
    for (state, expected) in [
        (Some("ready"), "Physics: jolt (Ready, 120 Hz)"),
        (Some("disabled"), "Physics: jolt (Disabled, 120 Hz)"),
        (Some("unavailable"), "Physics: jolt (Unavailable, 120 Hz)"),
        (None, "Physics: jolt (Unknown, 120 Hz)"),
    ] {
        let diagnostics = RuntimeDiagnosticsSnapshot {
            physics: RuntimePhysicsDiagnostics {
                available: true,
                backend_name: Some("jolt".to_string()),
                backend_status: state.map(|state| RuntimePhysicsBackendDiagnostics {
                    active_backend: Some("jolt".to_string()),
                    state: state.to_string(),
                    ..RuntimePhysicsBackendDiagnostics::default()
                }),
                fixed_hz: Some(120),
                error: None,
            },
            ..RuntimeDiagnosticsSnapshot::default()
        };

        assert_eq!(physics_status(&diagnostics), expected);
    }

    let unavailable = RuntimeDiagnosticsSnapshot {
        physics: RuntimePhysicsDiagnostics::unavailable("backend feature gate disabled"),
        ..RuntimeDiagnosticsSnapshot::default()
    };
    assert_eq!(
        physics_status(&unavailable),
        "Physics: unavailable (backend feature gate disabled)"
    );
}

#[test]
fn hybrid_gi_details_show_effective_profile_budgets_and_structured_fallback() {
    let diagnostics = RuntimeDiagnosticsSnapshot {
        render: RuntimeRenderDiagnostics {
            available: true,
            stats: Some(RenderStats {
                last_hybrid_gi_active_probe_count: 4,
                last_hybrid_gi_resolved_settings: Some(RenderHybridGiResolvedSettings {
                    mode: RenderHybridGiMode::DynamicOnly,
                    profile: RenderHybridGiProfile::IndoorStatic,
                    quality: RenderHybridGiQuality::High,
                    trace_budget: 64,
                    card_budget: 256,
                    voxel_budget: 64,
                    fallback_reason: Some(RenderHybridGiFallbackReason::BakedLightingUnavailable),
                }),
                ..RenderStats::default()
            }),
            ..RuntimeRenderDiagnostics::default()
        },
        ..RuntimeDiagnosticsSnapshot::default()
    };

    let items = detail_items(&diagnostics);
    assert!(items.contains(
        &"Hybrid GI effective: profile=indoor-static, mode=dynamic-only, quality=high".to_string()
    ));
    assert!(items.contains(&"Hybrid GI budgets: trace=64, cards=256, voxels=64".to_string()));
    assert!(items.contains(&"Hybrid GI fallback: baked-lighting-unavailable".to_string()));
}
